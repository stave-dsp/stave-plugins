//! The boundary between Anvil's panel and its amplifier design.
//!
//! Panel values are host state. [`AnvilDesign`] is the product's fixed
//! construction. [`resolve_engine_settings`] is the only place a player
//! control may map to more than one engine quantity.

use stave::params::{AnyParam, Ramp};
use stave::prelude::*;

use crate::params::AnvilParameters;

/// The player-facing state, in the units printed on the panel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanelControlValues {
    pub preamp_gain_db: f32,
    pub input_low_cut_hz: f32,
    pub bass_tone_position: f32,
    pub midrange_tone_position: f32,
    pub treble_tone_position: f32,
    pub presence_tone_db: f32,
    pub depth_tone_db: f32,
    pub topology_separation_amount: f32,
    pub power_pair_overdrive_db: f32,
    pub difference_tone_amount: f32,
    pub high_frequency_emphasis_amount: f32,
}

/// The panel as the processor renders it: each knob glides as its
/// param declares (one [`Smoother`] per knob).
pub struct Panel {
    preamp_gain: Smoother,
    input_low_cut: Smoother,
    bass_tone: Smoother,
    midrange_tone: Smoother,
    treble_tone: Smoother,
    presence_tone: Smoother,
    depth_tone: Smoother,
    topology_separation_amount: Smoother,
    power_pair_overdrive: Smoother,
    difference_tone: Smoother,
    high_frequency_emphasis: Smoother,
}

/// One block of the panel's glides.
pub struct PanelRamps<'s> {
    preamp_gain: Ramp<'s>,
    input_low_cut: Ramp<'s>,
    bass_tone: Ramp<'s>,
    midrange_tone: Ramp<'s>,
    treble_tone: Ramp<'s>,
    presence_tone: Ramp<'s>,
    depth_tone: Ramp<'s>,
    topology_separation_amount: Ramp<'s>,
    power_pair_overdrive: Ramp<'s>,
    difference_tone: Ramp<'s>,
    high_frequency_emphasis: Ramp<'s>,
}

/// The panel's values at one frame.
#[derive(Copy, Clone, Debug)]
pub struct PanelValues {
    pub preamp_gain: f32,
    pub input_low_cut: f32,
    pub bass_tone: f32,
    pub midrange_tone: f32,
    pub treble_tone: f32,
    pub presence_tone: f32,
    pub depth_tone: f32,
    pub topology_separation_amount: f32,
    pub power_pair_overdrive: f32,
    pub difference_tone: f32,
    pub high_frequency_emphasis: f32,
}

impl Panel {
    pub fn new(parameters: &AnvilParameters) -> Self {
        Self {
            preamp_gain: Smoother::new(&parameters.preamp_gain),
            input_low_cut: Smoother::new(&parameters.input_low_cut),
            bass_tone: Smoother::new(&parameters.bass_tone),
            midrange_tone: Smoother::new(&parameters.midrange_tone),
            treble_tone: Smoother::new(&parameters.treble_tone),
            presence_tone: Smoother::new(&parameters.presence_tone),
            depth_tone: Smoother::new(&parameters.depth_tone),
            topology_separation_amount: Smoother::new(&parameters.topology_separation_amount),
            power_pair_overdrive: Smoother::new(&parameters.power_pair_overdrive),
            difference_tone: Smoother::new(&parameters.difference_tone),
            high_frequency_emphasis: Smoother::new(&parameters.high_frequency_emphasis),
        }
    }

    pub fn prepare(&mut self, environment: &Environment) {
        self.preamp_gain.prepare(environment);
        self.input_low_cut.prepare(environment);
        self.bass_tone.prepare(environment);
        self.midrange_tone.prepare(environment);
        self.treble_tone.prepare(environment);
        self.presence_tone.prepare(environment);
        self.depth_tone.prepare(environment);
        self.topology_separation_amount.prepare(environment);
        self.power_pair_overdrive.prepare(environment);
        self.difference_tone.prepare(environment);
        self.high_frequency_emphasis.prepare(environment);
    }

    /// Jump every glide to its param's value.
    pub fn snap(&mut self, parameters: &AnvilParameters) {
        self.preamp_gain.snap(AnyParam::value(&parameters.preamp_gain));
        self.input_low_cut.snap(AnyParam::value(&parameters.input_low_cut));
        self.bass_tone.snap(AnyParam::value(&parameters.bass_tone));
        self.midrange_tone.snap(AnyParam::value(&parameters.midrange_tone));
        self.treble_tone.snap(AnyParam::value(&parameters.treble_tone));
        self.presence_tone.snap(AnyParam::value(&parameters.presence_tone));
        self.depth_tone.snap(AnyParam::value(&parameters.depth_tone));
        self.topology_separation_amount.snap(AnyParam::value(&parameters.topology_separation_amount));
        self.power_pair_overdrive.snap(AnyParam::value(&parameters.power_pair_overdrive));
        self.difference_tone.snap(AnyParam::value(&parameters.difference_tone));
        self.high_frequency_emphasis.snap(AnyParam::value(&parameters.high_frequency_emphasis));
    }

    pub fn ramp<'s>(&'s mut self, block: &Block<'s>, parameters: &AnvilParameters) -> PanelRamps<'s> {
        PanelRamps {
            preamp_gain: self.preamp_gain.ramp(block, &parameters.preamp_gain),
            input_low_cut: self.input_low_cut.ramp(block, &parameters.input_low_cut),
            bass_tone: self.bass_tone.ramp(block, &parameters.bass_tone),
            midrange_tone: self.midrange_tone.ramp(block, &parameters.midrange_tone),
            treble_tone: self.treble_tone.ramp(block, &parameters.treble_tone),
            presence_tone: self.presence_tone.ramp(block, &parameters.presence_tone),
            depth_tone: self.depth_tone.ramp(block, &parameters.depth_tone),
            topology_separation_amount: self.topology_separation_amount.ramp(block, &parameters.topology_separation_amount),
            power_pair_overdrive: self.power_pair_overdrive.ramp(block, &parameters.power_pair_overdrive),
            difference_tone: self.difference_tone.ramp(block, &parameters.difference_tone),
            high_frequency_emphasis: self.high_frequency_emphasis.ramp(block, &parameters.high_frequency_emphasis),
        }
    }
}

impl PanelRamps<'_> {
    /// The next frame's values (an endless per-frame stream, like
    /// [`Ramp::next`]).
    pub fn next(&mut self) -> PanelValues {
        PanelValues {
            preamp_gain: self.preamp_gain.next(),
            input_low_cut: self.input_low_cut.next(),
            bass_tone: self.bass_tone.next(),
            midrange_tone: self.midrange_tone.next(),
            treble_tone: self.treble_tone.next(),
            presence_tone: self.presence_tone.next(),
            depth_tone: self.depth_tone.next(),
            topology_separation_amount: self.topology_separation_amount.next(),
            power_pair_overdrive: self.power_pair_overdrive.next(),
            difference_tone: self.difference_tone.next(),
            high_frequency_emphasis: self.high_frequency_emphasis.next(),
        }
    }
}

impl From<PanelValues> for PanelControlValues {
    fn from(v: PanelValues) -> Self {
        Self {
            preamp_gain_db: v.preamp_gain,
            input_low_cut_hz: v.input_low_cut,
            bass_tone_position: v.bass_tone,
            midrange_tone_position: v.midrange_tone,
            treble_tone_position: v.treble_tone,
            presence_tone_db: v.presence_tone,
            depth_tone_db: v.depth_tone,
            topology_separation_amount: v.topology_separation_amount,
            power_pair_overdrive_db: v.power_pair_overdrive,
            difference_tone_amount: v.difference_tone,
            high_frequency_emphasis_amount: v.high_frequency_emphasis,
        }
    }
}

impl PanelControlValues {
    pub fn read(parameters: &AnvilParameters) -> Self {
        Self {
            preamp_gain_db: parameters.preamp_gain.value(),
            input_low_cut_hz: parameters.input_low_cut.value(),
            bass_tone_position: parameters.bass_tone.value(),
            midrange_tone_position: parameters.midrange_tone.value(),
            treble_tone_position: parameters.treble_tone.value(),
            presence_tone_db: parameters.presence_tone.value(),
            depth_tone_db: parameters.depth_tone.value(),
            topology_separation_amount: parameters.topology_separation_amount.value(),
            power_pair_overdrive_db: parameters.power_pair_overdrive.value(),
            difference_tone_amount: parameters.difference_tone.value(),
            high_frequency_emphasis_amount: parameters.high_frequency_emphasis.value(),
        }
    }
}

/// Non-automatable construction choices for this Anvil model.
///
/// This is ordinary Rust data rather than hidden host parameters. A future
/// Anvil model can persist a versioned design; other amplifiers can compose
/// the same DSP mechanisms without inheriting these choices.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnvilDesign {
    pub reference_input_level_db: f32,
    pub target_output_level_db: f32,
    pub power_pair_cold_bias_volts: f64,
    pub preamp_stage_count: usize,
    pub preamp_gain_offset_db: f32,
}

impl Default for AnvilDesign {
    fn default() -> Self {
        Self {
            reference_input_level_db: -12.0,
            target_output_level_db: -8.0,
            // The former PARALLEL AMPLIFIER default silently derived this as
            // 0.8 + 12 dB * 0.12. It is now a construction choice.
            power_pair_cold_bias_volts: 2.24,
            preamp_stage_count: 4,
            preamp_gain_offset_db: 6.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputSettings {
    pub highpass_cutoff_hz: f32,
    pub low_band_output_gain: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmplifierSettings {
    pub preamp_stage_count: usize,
    pub preamp_drive_db: f32,
    pub power_pair_overdrive_db: f32,
    pub topology_separation_amount: f32,
    pub reference_input_level_db: f32,
    pub blocking_high_shelf_depth: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToneSettings {
    pub bass_control_position: f32,
    pub midrange_control_position: f32,
    pub treble_control_position: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OutputSettings {
    pub presence_gain_db: f32,
    pub depth_gain_db: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EngineSettings {
    pub input: InputSettings,
    pub difference_tone_injection_amount: f32,
    pub amplifier: AmplifierSettings,
    pub tone: ToneSettings,
    pub output_conditioner: OutputSettings,
}

/// Resolve panel macros into objective section settings.
pub fn resolve_engine_settings(
    panel_controls: PanelControlValues,
    design: &AnvilDesign,
) -> EngineSettings {
    let input_low_cut_position =
        ((panel_controls.input_low_cut_hz / 40.0).ln() / (300.0f32 / 40.0).ln()).clamp(0.0, 1.0);
    EngineSettings {
        input: InputSettings {
            highpass_cutoff_hz: panel_controls.input_low_cut_hz,
            low_band_output_gain: 10f32
                .powf(-10.0 * input_low_cut_position * input_low_cut_position / 20.0),
        },
        difference_tone_injection_amount: panel_controls.difference_tone_amount,
        amplifier: AmplifierSettings {
            preamp_stage_count: design.preamp_stage_count,
            preamp_drive_db: design.preamp_gain_offset_db + panel_controls.preamp_gain_db * 0.5,
            power_pair_overdrive_db: panel_controls.power_pair_overdrive_db,
            topology_separation_amount: panel_controls.topology_separation_amount,
            reference_input_level_db: design.reference_input_level_db,
            blocking_high_shelf_depth: panel_controls.high_frequency_emphasis_amount,
        },
        tone: ToneSettings {
            bass_control_position: panel_controls.bass_tone_position,
            midrange_control_position: panel_controls.midrange_tone_position,
            treble_control_position: panel_controls.treble_tone_position,
        },
        output_conditioner: OutputSettings {
            presence_gain_db: panel_controls.presence_tone_db,
            depth_gain_db: panel_controls.depth_tone_db,
        },
    }
}

/// State-semantic application gate: first application fires; an equal value
/// does not. It keeps section comparison explicit without another framework.
#[derive(Clone, Debug)]
pub struct LastAppliedSettings<T> {
    value: Option<T>,
}

impl<T> LastAppliedSettings<T> {
    pub const fn new() -> Self {
        Self { value: None }
    }

    pub fn reset(&mut self) {
        self.value = None;
    }
}

impl<T: PartialEq + Copy> LastAppliedSettings<T> {
    pub fn has_changed(&mut self, value: T) -> bool {
        let changed = self.value != Some(value);
        self.value = Some(value);
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn representative_panel_controls() -> PanelControlValues {
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
            difference_tone_amount: 0.35,
            high_frequency_emphasis_amount: 0.0,
        }
    }

    #[test]
    fn controls_map_to_named_sections() {
        let settings =
            resolve_engine_settings(representative_panel_controls(), &AnvilDesign::default());
        assert_eq!(settings.input.highpass_cutoff_hz, 120.0);
        assert_eq!(settings.amplifier.preamp_drive_db, 15.0);
        assert_eq!(settings.amplifier.power_pair_overdrive_db, 12.0);
        assert_eq!(settings.difference_tone_injection_amount, 0.35);
        assert_eq!(settings.tone.midrange_control_position, 4.583_333);
        assert_eq!(settings.output_conditioner.presence_gain_db, 2.5);
    }

    #[test]
    fn tight_is_the_only_declared_macro() {
        let mut low_setting = representative_panel_controls();
        low_setting.input_low_cut_hz = 40.0;
        let mut high_setting = representative_panel_controls();
        high_setting.input_low_cut_hz = 300.0;
        let design = AnvilDesign::default();
        let low_settings = resolve_engine_settings(low_setting, &design);
        let high_settings = resolve_engine_settings(high_setting, &design);
        assert_eq!(low_settings.input.low_band_output_gain, 1.0);
        assert!((20.0 * high_settings.input.low_band_output_gain.log10() + 10.0).abs() < 1e-5);
        assert_eq!(low_settings.amplifier, high_settings.amplifier);
        assert_eq!(low_settings.tone, high_settings.tone);
        assert_eq!(
            low_settings.output_conditioner,
            high_settings.output_conditioner
        );
    }

    #[test]
    fn applied_uses_state_not_write_events() {
        let mut last_applied_settings = LastAppliedSettings::new();
        assert!(last_applied_settings.has_changed(3));
        assert!(!last_applied_settings.has_changed(3));
        assert!(last_applied_settings.has_changed(4));
        last_applied_settings.reset();
        assert!(last_applied_settings.has_changed(4));
    }
}
