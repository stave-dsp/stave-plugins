//! Anvil's product-specific amplifier composition.
//!
//! ```text
//! engine.rs              orchestration (resolve → apply → render)
//! parallel_amplifier.rs  blend between complete amplifier topologies
//! parallel_amplifier/    nonlinear branch and split-band topology
//! output_conditioner.rs  post-amplifier response and bandwidth
//! ```
//! Path per lane: input high-pass → difference-tone generation → parallel
//! amplifier → high-band limiting → passive tone stack → output conditioner.
//! Reusable mechanisms live in stave-dsp; this topology stays concrete because
//! it is Anvil's product design.

use stave::dsp::Slew;
use stave::prelude::*;

mod difference_tone;
mod high_band_limiter;
mod output_conditioner;
mod parallel_amplifier;
mod tone_section;

use difference_tone::DifferenceToneGenerator;
use high_band_limiter::HighBandLimiter;
use output_conditioner::OutputConditioner;
use parallel_amplifier::{ParallelAmplifier, SettledSupply};
pub use parallel_amplifier::RcCoupledTriodeCascade;
use tone_section::ToneSection;

use crate::controls::{
    resolve_engine_settings, AmplifierSettings, AnvilDesign, InputSettings, LastAppliedSettings,
    OutputSettings, Panel, PanelControlValues, ToneSettings,
};
use crate::params::AnvilParameters;

#[derive(Clone)]
struct ProcessingLane {
    input_highpass_filter: Biquad,
    difference_tone_generator: DifferenceToneGenerator,
    /// Post-parallel amplifier slew (field-ratified in the SMOOTH pair): big
    /// fast swings round, small ones pass — level-dependent
    /// transient smoothing, the Miller-physics cousin.
    slew_limiter: Slew,
    previous_amplifier_output: f32,
    // THE OVERSAMPLER IS GONE (field-ratified 2026-08-06): measured
    // at the ratified voice, the 2x kernel bought 2 dB of alias
    // floor (-45.8 vs -43.8 dBc worst inharmonic, 3.5 kHz probe at
    // 44.1k) for ~45% of the plugin's CPU — the ADAA curves ARE
    // the antialiasing. The parallel amplifier runs at the host rate; zero
    // latency came with it.
    parallel_amplifier: ParallelAmplifier,
    high_band_limiter: HighBandLimiter,
    tone_section: ToneSection,
    output_conditioner: OutputConditioner,
}

/// REGEN (movement rail): a whisper of the parallel amplifier's own output fed
/// back to its input — the guitarist standing near the cab, dosed.
/// The one mechanism the field's ear kept naming ("feels like
/// feedback"). <= -99 = off, bit-identical.
const AMPLIFIER_FEEDBACK_DB: f32 = -100.0;

/// Anvil's observation points, out to the bench and the face.
#[derive(Taps)]
pub struct AnvilTaps {
    /// The compile timestamp (build.rs): a bench measurement without a
    /// fresh stamp is fiction — the poisoned-measurement day's law.
    pub build: Tap,
    pub classic_out_db: Tap,
    pub low_out_db: Tap,
    pub high_out_db: Tap,
    pub pair_in_db: Tap,
    pub draw: Tap,
    pub v_min: Tap,
    pub v_max: Tap,
    pub draw_mean: Tap,
}

/// The plugin: the panel, its glides, and the amplifier they drive.
pub struct Anvil {
    taps: AnvilTaps,
    parameters: Arc<AnvilParameters>,
    amplifier: Amplifier,
    /// The amplifier as `prepare` settled it: `reset` copies it back
    /// rather than re-settling on the audio thread.
    settled: Amplifier,
    panel: Panel,
    output_level: Smoother,
}

/// The panel re-resolves into the amplifier this often while it moves:
/// a 20 ms glide steps under −30 dB per interval.
const CONTROL_INTERVAL: usize = 16;

/// The lanes and the settings last applied to them.
#[derive(Clone)]
struct Amplifier {
    sample_rate: f32,
    lanes: PerChannel<ProcessingLane, 2>,
    design: AnvilDesign,
    last_input_settings: LastAppliedSettings<InputSettings>,
    last_difference_tone_injection_amount: LastAppliedSettings<f32>,
    last_amplifier_settings: LastAppliedSettings<AmplifierSettings>,
    last_tone_settings: LastAppliedSettings<ToneSettings>,
    last_output_settings: LastAppliedSettings<OutputSettings>,
    /// Computed makeup: the parallel amplifier reports its small-signal
    /// gain and saturated ceiling, so the expected output for the
    /// reference input is min(input + gain, ceiling), and makeup is the
    /// target minus that.
    makeup_gain: f32,
}

impl Amplifier {
    /// Prepare every lane at `sample_rate`, then [`settle`](Self::settle)
    /// it for `panel`. The lanes are built alike and settle on silence,
    /// so they reach the same state: the first is prepared and the
    /// others are copies of it.
    fn prepare(&mut self, sample_rate: f32, panel: PanelControlValues) {
        self.sample_rate = sample_rate;
        let settled_supply = SettledSupply::at(sample_rate);
        let (lane, others) = self.lanes.pair_mut();
        // Cleared first: the operating point must not depend on what
        // the lane last played.
        lane.input_highpass_filter.reset();
        lane.difference_tone_generator.reset();
        lane.slew_limiter.reset();
        lane.parallel_amplifier.reset();
        lane.high_band_limiter.reset();
        lane.tone_section.reset();
        lane.output_conditioner.reset();
        lane.input_highpass_filter.prepare(sample_rate);
        lane.difference_tone_generator.prepare(sample_rate);
        lane.slew_limiter.prepare(sample_rate);
        lane.parallel_amplifier.prepare(
            sample_rate,
            self.design.power_pair_cold_bias_volts,
            &settled_supply,
        );
        lane.high_band_limiter.prepare(sample_rate);
        lane.tone_section.prepare(sample_rate);
        lane.output_conditioner.prepare(sample_rate);
        lane.previous_amplifier_output = 0.0;
        others.clone_from(lane);
        self.settle(panel);
    }

    /// Voice every lane for `panel` and settle it at its real operating
    /// point (bias, blocking, cathode, supply), so silence in is silence
    /// out from the first block: run until the output stays under 1e-5
    /// for 256 samples, at most one second. Every lane is voiced alike
    /// and fed the same silence, so one is run and copied to the others.
    fn settle(&mut self, panel: PanelControlValues) {
        self.last_input_settings.reset();
        self.last_difference_tone_injection_amount.reset();
        self.last_amplifier_settings.reset();
        self.last_tone_settings.reset();
        self.last_output_settings.reset();
        self.apply(panel);
        let sample_rate = self.sample_rate;
        let (lane, others) = self.lanes.pair_mut();
        let mut quiet = 0u32;
        for _ in 0..sample_rate as usize {
            quiet = if process_lane_sample(lane, 0.0).abs() < 1e-5 {
                quiet + 1
            } else {
                0
            };
            if quiet >= 256 {
                break;
            }
        }
        others.clone_from(lane);
    }

    fn apply(&mut self, panel: PanelControlValues) {
        let settings = resolve_engine_settings(panel, &self.design);
        let input_settings_changed = self.last_input_settings.has_changed(settings.input);
        let difference_tone_settings_changed = self
            .last_difference_tone_injection_amount
            .has_changed(settings.difference_tone_injection_amount);
        let amplifier_settings_changed =
            self.last_amplifier_settings.has_changed(settings.amplifier);
        let tone_settings_changed = self.last_tone_settings.has_changed(settings.tone);
        let output_settings_changed = self
            .last_output_settings
            .has_changed(settings.output_conditioner);

        for processing_lane in self.lanes.iter_mut() {
            if input_settings_changed {
                processing_lane.input_highpass_filter.set(FilterSpec::Highpass {
                    hz: settings.input.highpass_cutoff_hz,
                    q: 0.71,
                });
            }
            if difference_tone_settings_changed {
                processing_lane
                    .difference_tone_generator
                    .set_injection_amount(settings.difference_tone_injection_amount);
            }
            if input_settings_changed || amplifier_settings_changed {
                processing_lane
                    .parallel_amplifier
                    .apply_settings(settings.amplifier, settings.input);
            }
            if tone_settings_changed {
                processing_lane.tone_section.apply_settings(settings.tone);
            }
            if output_settings_changed {
                processing_lane
                    .output_conditioner
                    .apply_settings(settings.output_conditioner);
            }
        }
        if amplifier_settings_changed || tone_settings_changed {
            let expected_amplifier_output_db = self.lanes[0]
                .parallel_amplifier
                .expected_output_level_db(self.design.reference_input_level_db);
            let tone_section_level_compensation_gain =
                self.lanes[0].tone_section.level_compensation_gain();
            self.makeup_gain = db_to_amp(
                (self.design.target_output_level_db - expected_amplifier_output_db)
                    .clamp(-40.0, 60.0),
            ) * tone_section_level_compensation_gain;
        }
    }
}

#[inline]
fn process_lane_sample(processing_lane: &mut ProcessingLane, input_sample: f32) -> f32 {
    let conditioned_input = processing_lane.input_highpass_filter.tick(input_sample)
        + if AMPLIFIER_FEEDBACK_DB > -99.0 {
            db_to_amp(AMPLIFIER_FEEDBACK_DB) * processing_lane.previous_amplifier_output
        } else {
            0.0
        };
    let generated_input = processing_lane
        .difference_tone_generator
        .process_sample(conditioned_input);
    let unbounded_amplifier_output = processing_lane
        .parallel_amplifier
        .process_sample(generated_input);
    let amplifier_output = processing_lane
        .high_band_limiter
        .process_sample(unbounded_amplifier_output);
    processing_lane.previous_amplifier_output = amplifier_output;
    let slew_limited_output =
        processing_lane.slew_limiter.tick(amplifier_output);
    let tone_stack_output = processing_lane
        .tone_section
        .process_sample(slew_limited_output);
    processing_lane
        .output_conditioner
        .process_sample(tone_stack_output)
}

impl Plugin for Anvil {
    const INFO: Info = Info {
        id: "com.ryanbahan.stave.anvil.vst3",
        name: "Stave Anvil",
        vendor: "Ryan Bahan",
        url: "https://github.com/stave-dsp/stave",
        description: "The impossible metalcore head.",
        features: &[Feature::AudioEffect, Feature::Stereo],
        ..Info::EFFECT
    };
    /// 1: the tone stack's bass/mid/treble moved from dB to knob
    /// positions (0–10).
    const STATE_VERSION: u32 = 1;
    type Params = AnvilParameters;

    fn new(parameters: &Arc<AnvilParameters>, telemetry: &Telemetry) -> Self {
        let taps = AnvilTaps::new(telemetry);
        // The render self-identifies: `build` = compile timestamp.
        taps.build
            .set(env!("STAVE_BUILD_STAMP").parse::<f64>().unwrap_or(0.0));
        // Every lane is built alike: one is built (its tube tables are
        // the cost) and the others are copies of it.
        let lane = ProcessingLane {
            input_highpass_filter: Biquad::passthrough(),
            difference_tone_generator: DifferenceToneGenerator::new(),
            slew_limiter: Slew::new(8.0, 8.0),
            previous_amplifier_output: 0.0,
            parallel_amplifier: ParallelAmplifier::new(),
            high_band_limiter: HighBandLimiter::new(),
            tone_section: ToneSection::new(),
            output_conditioner: OutputConditioner::new(),
        };
        let amplifier = Amplifier {
                sample_rate: 48_000.0,
                lanes: PerChannel::new(|_| lane.clone()),
                design: AnvilDesign::default(),
                last_input_settings: LastAppliedSettings::new(),
                last_difference_tone_injection_amount: LastAppliedSettings::new(),
                last_amplifier_settings: LastAppliedSettings::new(),
                last_tone_settings: LastAppliedSettings::new(),
                last_output_settings: LastAppliedSettings::new(),
                makeup_gain: 1.0,
            };
        Self {
            taps,
            settled: amplifier.clone(),
            amplifier,
            panel: Panel::new(parameters),
            output_level: Smoother::new(&parameters.output_level),
            parameters: parameters.clone(),
        }
    }

    fn prepare(&mut self, environment: &Environment) {
        self.output_level.prepare(environment);
        self.panel.prepare(environment);
        self.amplifier.prepare(
            environment.sample_rate,
            PanelControlValues::read(&self.parameters),
        );
        self.settled.clone_from(&self.amplifier);
    }

    /// Back to the state `prepare` left: the panel snaps to the params
    /// and the amplifier is the settled copy `prepare` kept (no
    /// simulation, no allocation); the next block re-voices it for the
    /// panel as it is now.
    fn reset(&mut self) {
        self.output_level
            .snap(AnyParam::value(&self.parameters.output_level));
        self.panel.snap(&self.parameters);
        self.amplifier.clone_from(&self.settled);
    }

    fn process(&mut self, block: &mut Block<'_>) {
        let parameters = &self.parameters;
        let mut panel = self.panel.ramp(block, parameters);
        let mut output_level_values = self.output_level.ramp(block, &parameters.output_level);
        let amplifier = &mut self.amplifier;
        let audio_lanes = block.lanes::<2>();
        let active_channel_count = audio_lanes.active().min(2);
        audio_lanes.run(|audio_frame| {
            let panel_values = panel.next();
            if audio_frame.index() % CONTROL_INTERVAL == 0 {
                amplifier.apply(panel_values.into());
            }
            let final_output_gain =
                db_to_amp(output_level_values.next()) * amplifier.makeup_gain * 0.5;
            for channel_index in 0..active_channel_count {
                let output_sample = process_lane_sample(
                    &mut amplifier.lanes[channel_index],
                    audio_frame.input(channel_index),
                ) * final_output_gain;
                audio_frame.write(channel_index, output_sample);
            }
        });
        drop(panel);
        drop(output_level_values);
        let measurements = self.amplifier.lanes[0]
            .parallel_amplifier
            .take_measurements();
        let amplitude_to_decibels = |amplitude: f32| f64::from(amp_to_db(amplitude.max(1e-9)));
        let taps = &self.taps;
        taps.classic_out_db
            .set(amplitude_to_decibels(measurements.full_band_output_peak));
        taps.low_out_db
            .set(amplitude_to_decibels(measurements.low_band_output_peak));
        taps.high_out_db
            .set(amplitude_to_decibels(measurements.high_band_output_peak));
        taps.pair_in_db
            .set(amplitude_to_decibels(measurements.full_band_power_pair_input_peak));
        taps.draw
            .set(f64::from(measurements.full_band_power_pair_draw_peak));
        taps.v_min
            .set(f64::from(measurements.full_band_supply_voltage_minimum));
        taps.v_max
            .set(f64::from(measurements.full_band_supply_voltage_maximum));
        taps.draw_mean.set(f64::from(
            measurements.full_band_power_pair_draw_sum
                / measurements.full_band_power_pair_draw_sample_count.max(1.0),
        ));
    }

    fn face(parameters: &AnvilParameters) -> Option<Face> {
        Some(crate::editor::face(parameters))
    }

    fn presets(parameters: &AnvilParameters) -> Vec<Preset> {
        crate::presets::build_presets(parameters)
    }

    fn migrations() -> Migrations {
        fn legacy_six_decibel_tone_value_to_position(value: f64) -> f64 {
            (value + 6.0) * (10.0 / 12.0)
        }

        fn legacy_midrange_tone_value_to_position(value: f64) -> f64 {
            (value + 8.0) * (10.0 / 12.0)
        }

        Migrations::default()
            .rescaled_before(1, ParamId(6), legacy_six_decibel_tone_value_to_position)
            .rescaled_before(1, ParamId(7), legacy_midrange_tone_value_to_position)
            .rescaled_before(1, ParamId(8), legacy_six_decibel_tone_value_to_position)
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    fn measure_nanoseconds_per_sample(mut process_sample: impl FnMut(f32) -> f32) -> f64 {
        const SAMPLES: usize = 96_000;
        let start = std::time::Instant::now();
        let mut output_sample = 0.0;
        for sample_index in 0..SAMPLES {
            let input_sample =
                (core::f32::consts::TAU * 220.0 * sample_index as f32 / 48_000.0).sin() * 0.25;
            output_sample = process_sample(std::hint::black_box(input_sample));
        }
        std::hint::black_box(output_sample);
        start.elapsed().as_secs_f64() * 1e9 / SAMPLES as f64
    }

    #[test]
    fn component_costs_for_the_record() {
        let design = AnvilDesign::default();
        let settings = resolve_engine_settings(
            PanelControlValues {
                preamp_gain_db: 18.0,
                input_low_cut_hz: 120.0,
                bass_tone_position: 6.25,
                midrange_tone_position: 4.583_333,
                treble_tone_position: 7.5,
                presence_tone_db: 2.5,
                depth_tone_db: 2.5,
                topology_separation_amount: 0.5,
                power_pair_overdrive_db: 12.0,
                difference_tone_amount: 0.0,
                high_frequency_emphasis_amount: 0.0,
            },
            &design,
        );
        let mut amplifier = ParallelAmplifier::new();
        amplifier.prepare(
            48_000.0,
            design.power_pair_cold_bias_volts,
            &SettledSupply::at(48_000.0),
        );
        amplifier.apply_settings(settings.amplifier, settings.input);
        let mut tone_section = ToneSection::new();
        tone_section.prepare(48_000.0);
        tone_section.apply_settings(settings.tone);
        let mut output_conditioner = OutputConditioner::new();
        output_conditioner.prepare(48_000.0);
        output_conditioner.apply_settings(settings.output_conditioner);

        let amplifier_nanoseconds_per_sample =
            measure_nanoseconds_per_sample(move |input_sample| {
                amplifier.process_sample(input_sample)
            });
        let tone_section_nanoseconds_per_sample =
            measure_nanoseconds_per_sample(move |input_sample| {
                tone_section.process_sample(input_sample)
            });
        let output_conditioner_nanoseconds_per_sample =
            measure_nanoseconds_per_sample(move |input_sample| {
                output_conditioner.process_sample(input_sample)
            });
        eprintln!(
            "anvil component ns/sample: amplifier={amplifier_nanoseconds_per_sample:.1}, \
             tone={tone_section_nanoseconds_per_sample:.1}, \
             output={output_conditioner_nanoseconds_per_sample:.1}"
        );
    }
}
