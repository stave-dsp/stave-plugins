//! Anvil's three-band nonlinear amplifier topology.
//!
//! Frequency separation, branch isolation, three nonlinear amplifier
//! branches, and their linear recombination live here as one complete
//! topology.

use stave::dsp::LinkwitzRiley4;
use stave::prelude::*;

use super::branch::{AmplifierBranch, BranchDesign};
use crate::controls::{AmplifierSettings, InputSettings};

const LOW_CROSSOVER_HZ: f32 = 250.0;
const HIGH_CROSSOVER_HZ: f32 = 650.0;
const HIGH_BAND_OUTPUT_TRIM_DB: f32 = -2.5;
const LOW_MID_BAND_OUTPUT_GAIN: f32 = 1.5;

struct FrequencyBandSamples {
    low_band: f32,
    low_mid_band: f32,
    high_band: f32,
}

struct ThreeWayCrossover {
    low_crossover_lowpass: LinkwitzRiley4,
    low_crossover_highpass: LinkwitzRiley4,
    high_crossover_lowpass: LinkwitzRiley4,
    high_crossover_highpass: LinkwitzRiley4,
    low_band_phase_compensation_lowpass: LinkwitzRiley4,
    low_band_phase_compensation_highpass: LinkwitzRiley4,
}

impl ThreeWayCrossover {
    fn new() -> Self {
        Self {
            low_crossover_lowpass: LinkwitzRiley4::lowpass(LOW_CROSSOVER_HZ),
            low_crossover_highpass: LinkwitzRiley4::highpass(LOW_CROSSOVER_HZ),
            high_crossover_lowpass: LinkwitzRiley4::lowpass(HIGH_CROSSOVER_HZ),
            high_crossover_highpass: LinkwitzRiley4::highpass(HIGH_CROSSOVER_HZ),
            low_band_phase_compensation_lowpass: LinkwitzRiley4::lowpass(HIGH_CROSSOVER_HZ),
            low_band_phase_compensation_highpass: LinkwitzRiley4::highpass(HIGH_CROSSOVER_HZ),
        }
    }

    fn prepare(&mut self, sample_rate: f32) {
        self.low_crossover_lowpass.prepare(sample_rate);
        self.low_crossover_highpass.prepare(sample_rate);
        self.high_crossover_lowpass.prepare(sample_rate);
        self.high_crossover_highpass.prepare(sample_rate);
        self.low_band_phase_compensation_lowpass.prepare(sample_rate);
        self.low_band_phase_compensation_highpass.prepare(sample_rate);
    }

    fn reset(&mut self) {
        self.low_crossover_lowpass.reset();
        self.low_crossover_highpass.reset();
        self.high_crossover_lowpass.reset();
        self.high_crossover_highpass.reset();
        self.low_band_phase_compensation_lowpass.reset();
        self.low_band_phase_compensation_highpass.reset();
    }

    #[inline]
    fn split_sample(&mut self, input_sample: f32) -> FrequencyBandSamples {
        let low_band = self.low_crossover_lowpass.tick(input_sample);
        let upper_bands = self.low_crossover_highpass.tick(input_sample);
        FrequencyBandSamples {
            low_band,
            low_mid_band: self.high_crossover_lowpass.tick(upper_bands),
            high_band: self.high_crossover_highpass.tick(upper_bands),
        }
    }

    #[inline]
    fn compensate_low_band_phase(&mut self, low_band_sample: f32) -> f32 {
        self.low_band_phase_compensation_lowpass
            .tick(low_band_sample)
            + self
                .low_band_phase_compensation_highpass
                .tick(low_band_sample)
    }
}

/// Matched input and output filtering for a deeply saturated branch.
///
/// The input guard prevents out-of-band content from creating
/// intermodulation products. The output guard confines generated harmonics so
/// the high-band branch remains the owner of high-frequency identity.
struct DeepBranchIsolationFilter {
    input_guard: Biquad,
    first_output_guard: Biquad,
    second_output_guard: Biquad,
    third_output_guard: Biquad,
}

impl DeepBranchIsolationFilter {
    fn new() -> Self {
        Self {
            input_guard: Biquad::new(FilterSpec::Lowpass { hz: 700.0, q: 0.6 }),
            first_output_guard: Biquad::new(FilterSpec::Lowpass {
                hz: 800.0,
                q: 0.518,
            }),
            second_output_guard: Biquad::new(FilterSpec::Lowpass {
                hz: 800.0,
                q: 0.707,
            }),
            third_output_guard: Biquad::new(FilterSpec::Lowpass {
                hz: 800.0,
                q: 1.932,
            }),
        }
    }

    fn prepare(&mut self, sample_rate: f32) {
        self.input_guard.prepare(sample_rate);
        self.first_output_guard.prepare(sample_rate);
        self.second_output_guard.prepare(sample_rate);
        self.third_output_guard.prepare(sample_rate);
    }

    fn reset(&mut self) {
        self.input_guard.reset();
        self.first_output_guard.reset();
        self.second_output_guard.reset();
        self.third_output_guard.reset();
    }

    #[inline]
    fn blend_input(&mut self, input_sample: f32, isolation_amount: f32) -> f32 {
        if isolation_amount <= 0.0 {
            input_sample
        } else {
            let filtered_input = self.input_guard.tick(input_sample);
            input_sample + (filtered_input - input_sample) * isolation_amount
        }
    }

    #[inline]
    fn blend_output(&mut self, unguarded_output: f32, isolation_amount: f32) -> f32 {
        if isolation_amount <= 0.0 {
            unguarded_output
        } else {
            let filtered_output = self.third_output_guard.tick(
                self.second_output_guard
                    .tick(self.first_output_guard.tick(unguarded_output)),
            );
            unguarded_output + (filtered_output - unguarded_output) * isolation_amount
        }
    }
}

#[derive(Default, Clone, Copy)]
pub(super) struct SplitBandMeasurements {
    pub(super) low_band_output_peak: f32,
    pub(super) high_band_output_peak: f32,
}

pub(super) struct SplitBandAmplifier {
    crossover: ThreeWayCrossover,
    low_band_isolation_filter: DeepBranchIsolationFilter,
    low_mid_band_isolation_filter: DeepBranchIsolationFilter,
    low_band_branch: AmplifierBranch,
    low_mid_band_branch: AmplifierBranch,
    high_band_branch: AmplifierBranch,
    isolation_amount: f32,
    low_band_output_gain: f32,
    measurements: SplitBandMeasurements,
}

impl SplitBandAmplifier {
    pub(super) fn new() -> Self {
        Self {
            crossover: ThreeWayCrossover::new(),
            low_band_isolation_filter: DeepBranchIsolationFilter::new(),
            low_mid_band_isolation_filter: DeepBranchIsolationFilter::new(),
            low_band_branch: AmplifierBranch::new(BranchDesign::low_band()),
            low_mid_band_branch: AmplifierBranch::new(BranchDesign::low_mid_band()),
            high_band_branch: AmplifierBranch::new(BranchDesign::high_band()),
            isolation_amount: 0.0,
            low_band_output_gain: 1.0,
            measurements: SplitBandMeasurements::default(),
        }
    }

    pub(super) fn prepare(&mut self, sample_rate: f32, power_pair_cold_bias_volts: f64) {
        self.crossover.prepare(sample_rate);
        self.low_band_isolation_filter.prepare(sample_rate);
        self.low_mid_band_isolation_filter.prepare(sample_rate);
        self.low_band_branch
            .prepare(sample_rate, power_pair_cold_bias_volts);
        self.low_mid_band_branch
            .prepare(sample_rate, power_pair_cold_bias_volts);
        self.high_band_branch
            .prepare(sample_rate, power_pair_cold_bias_volts);
        self.measurements = SplitBandMeasurements::default();
    }

    pub(super) fn reset(&mut self) {
        self.crossover.reset();
        self.low_band_isolation_filter.reset();
        self.low_mid_band_isolation_filter.reset();
        self.low_band_branch.reset();
        self.low_mid_band_branch.reset();
        self.high_band_branch.reset();
        self.measurements = SplitBandMeasurements::default();
    }

    pub(super) fn apply_settings(
        &mut self,
        amplifier_settings: AmplifierSettings,
        input_settings: InputSettings,
        branch_drive_db: f32,
    ) {
        self.isolation_amount =
            ((amplifier_settings.topology_separation_amount - 0.75) / 0.15).clamp(0.0, 1.0);
        self.low_band_output_gain = input_settings.low_band_output_gain;
        self.high_band_branch
            .set_blocking_high_shelf_depth(amplifier_settings.blocking_high_shelf_depth);

        self.high_band_branch.apply_live_settings(
            amplifier_settings.preamp_stage_count,
            branch_drive_db,
            amplifier_settings.power_pair_overdrive_db,
            amplifier_settings.reference_input_level_db - 3.0,
        );
        self.low_band_branch.apply_live_settings(
            amplifier_settings.preamp_stage_count,
            branch_drive_db,
            amplifier_settings.power_pair_overdrive_db,
            amplifier_settings.reference_input_level_db - 3.0,
        );
        self.low_mid_band_branch.apply_live_settings(
            amplifier_settings.preamp_stage_count,
            branch_drive_db,
            amplifier_settings.power_pair_overdrive_db,
            amplifier_settings.reference_input_level_db - 6.0,
        );
    }

    #[inline]
    pub(super) fn process_sample(&mut self, input_sample: f32) -> f32 {
        let frequency_bands = self.crossover.split_sample(input_sample);

        let guarded_low_band_input = self
            .low_band_isolation_filter
            .blend_input(frequency_bands.low_band, self.isolation_amount);
        let unguarded_low_band_output = self.low_band_branch.process_sample(guarded_low_band_input);
        let guarded_low_band_output = self
            .low_band_isolation_filter
            .blend_output(unguarded_low_band_output, self.isolation_amount);
        let phase_compensated_low_band_output = self
            .crossover
            .compensate_low_band_phase(guarded_low_band_output);

        let guarded_low_mid_band_input = self
            .low_mid_band_isolation_filter
            .blend_input(frequency_bands.low_mid_band, self.isolation_amount);
        let unguarded_low_mid_band_output = self
            .low_mid_band_branch
            .process_sample(guarded_low_mid_band_input);
        let low_mid_band_output = self
            .low_mid_band_isolation_filter
            .blend_output(unguarded_low_mid_band_output, self.isolation_amount);

        let high_band_output = self
            .high_band_branch
            .process_sample(frequency_bands.high_band);

        self.measurements.low_band_output_peak = self
            .measurements
            .low_band_output_peak
            .max(phase_compensated_low_band_output.abs());
        self.measurements.high_band_output_peak = self
            .measurements
            .high_band_output_peak
            .max(high_band_output.abs());

        self.low_band_output_gain * phase_compensated_low_band_output
            + LOW_MID_BAND_OUTPUT_GAIN * low_mid_band_output
            + db_to_amp(HIGH_BAND_OUTPUT_TRIM_DB) * high_band_output
    }

    pub(super) fn take_measurements(&mut self) -> SplitBandMeasurements {
        let measurements = self.measurements;
        self.measurements = SplitBandMeasurements::default();
        measurements
    }
}
