//! Every check Opal owes (validation, no allocation while every knob
//! sweeps under a note storm, perf, the face's laws, host behaviour),
//! from one line. Perf baselines 2026-07-07 (Apple Silicon, ns/frame,
//! an 8-note chord): INIT CHORD ~510, FULL STACK (all four ops +
//! feedback) ~637 — ~3% of budget for 8 voices × 4 operators × 4
//! envelopes, all per-sample f64 sin.

stave_test::plugin_checks!(stave_opal_vst3::Opal, {
    perf: vec![
        PerfCase { name: "INIT CHORD", set: &[] },
        PerfCase {
            name: "FULL STACK",
            set: &[("algo", 0.0), ("level 2", 1.0), ("level 3", 1.0), ("level 4", 1.0), ("feedback", 0.9)],
        },
    ],
});
