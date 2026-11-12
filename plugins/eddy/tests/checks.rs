//! Every check Eddy owes (validation, no allocation, perf, the face's
//! laws, host behaviour), from one line. Perf baselines after the
//! sweep-driven optimization pass (2026-07-06, Apple Silicon,
//! ns/frame): CLASSIC ~50, CIRCUIT ~95, SHIFT+PLATE ~85, PRISM under
//! the field ~245 (was 378 — lane-interleaved cascade + guard-band
//! ticking; the splice search dropped 70% the same day by buffering
//! its span instead of re-reading through the cubic interpolator).

stave_test::plugin_checks!(stave_eddy_vst3::Eddy, {
    perf: vec![
        PerfCase { name: "CLASSIC", set: &[("feedback", 0.8), ("wet", 1.0)] },
        PerfCase {
            name: "CIRCUIT tape hot",
            set: &[("feedback", 0.8), ("algo", 0.0), ("circuit", 2.0), ("heat", 1.0)],
        },
        PerfCase {
            name: "SHIFT + plate field",
            set: &[
                ("feedback", 0.8),
                ("algo", 2.0),
                ("shift", 12.0),
                ("diffuse", 1.0),
                ("hold", 0.9),
                ("bleed", 0.5),
            ],
        },
        PerfCase {
            name: "PRISM full smear",
            set: &[("feedback", 0.8), ("diffusion", 1.0), ("diffuse", 1.0), ("hold", 0.9), ("smear", 1.0)],
        },
    ],
});
