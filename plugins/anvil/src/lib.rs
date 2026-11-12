//! Stave Anvil. **Start here**: this file is the plugin's
//! root; everything below is one click away.
//!
//! ```text
//! lib.rs      ← you are here: the modules and the export
//! params.rs   every knob, with its forever-stable id
//! editor.rs   the face, a pure function of params
//! presets.rs  factory presets
//! controls.rs the panel → amplifier-design boundary
//! engine.rs   `Anvil` and its `impl Plugin` (identity, state migrations,
//!               processing) over the engine/ components
//! tests/      checks.rs (every generic check, one line), the editor,
//!               and the LAWS (properties.rs, rig_properties.rs —
//!               write the first one BEFORE the DSP; Tilt tells you why)
//! ```
//!
//! STATUS: **R1 — RHYTHM**. Two complete rigs
//! crossfaded by FOCUS: one honest tube path, and the impossible
//! split where the chug cannot touch the teeth. Five laws green.

mod controls;
mod editor;
mod engine;
mod params;
mod presets;

pub use engine::Anvil;
/// For the workspace conformance suite (perf profile, rate invariance).
#[doc(hidden)]
pub use engine::RcCoupledTriodeCascade;
pub use params::AnvilParameters;

stave::export!(Anvil, vst3_class_ids: {
    component: [0x5A908C88, 0x84EC0CD9, 0x1727EBCC, 0x38EB1EFE],
    controller: [0xE2F5272C, 0x4B153E7C, 0xB777CA10, 0x96D452A6],
});

#[cfg(test)]
mod tests {
    use stave::prelude::*;
    use stave::state::{load, save, StateStore};

    use crate::AnvilParameters;

    #[derive(Params)]
    struct LegacyToneParameters {
        bass: FloatParam,
        midrange: FloatParam,
        treble: FloatParam,
    }

    fn legacy_tone_parameters() -> LegacyToneParameters {
        LegacyToneParameters {
            bass: FloatParam::new(ParamId(6), "bass", Range::linear(-6.0, 6.0)).default(1.5),
            midrange: FloatParam::new(ParamId(7), "mid", Range::linear(-8.0, 4.0)).default(-2.5),
            treble: FloatParam::new(ParamId(8), "treble", Range::linear(-6.0, 6.0)).default(3.0),
        }
    }

    #[test]
    fn legacy_tone_state_keeps_the_same_physical_knob_positions() {
        let legacy_parameters = legacy_tone_parameters();
        let legacy_state = save(&legacy_parameters, 0, &[], &StateStore::default());
        let current_parameters = AnvilParameters::default();
        load(
            &legacy_state,
            &current_parameters,
            &<crate::Anvil as Plugin>::migrations(),
            &mut StateStore::default(),
        )
        .unwrap();

        assert_eq!((&current_parameters.bass_tone as &dyn AnyParam).value(), 6.25);
        assert!(((&current_parameters.midrange_tone as &dyn AnyParam).value() - 4.583_333_333_333_333).abs() < 1e-12);
        assert_eq!((&current_parameters.treble_tone as &dyn AnyParam).value(), 7.5);
    }

    #[test]
    fn current_tone_state_is_not_migrated_twice() {
        let source_parameters = AnvilParameters::default();
        (&source_parameters.bass_tone as &dyn AnyParam).set_value(8.0);
        (&source_parameters.midrange_tone as &dyn AnyParam).set_value(3.0);
        (&source_parameters.treble_tone as &dyn AnyParam).set_value(6.0);
        let current_state = save(
            &source_parameters,
            <crate::Anvil as Plugin>::STATE_VERSION,
            &[],
            &StateStore::default(),
        );

        let loaded_parameters = AnvilParameters::default();
        load(
            &current_state,
            &loaded_parameters,
            &<crate::Anvil as Plugin>::migrations(),
            &mut StateStore::default(),
        )
        .unwrap();

        assert_eq!((&loaded_parameters.bass_tone as &dyn AnyParam).value(), 8.0);
        assert_eq!((&loaded_parameters.midrange_tone as &dyn AnyParam).value(), 3.0);
        assert_eq!((&loaded_parameters.treble_tone as &dyn AnyParam).value(), 6.0);
    }
}
