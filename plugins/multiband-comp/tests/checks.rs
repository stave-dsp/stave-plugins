//! Every check Multiband owes (validation, no allocation, perf, the
//! face's laws, host behaviour), from one line. Perf baseline 2026-10-13
//! (Apple Silicon, ns/frame): DEFAULT ~120, DRIVEN ~770 (saturators
//! engaged), OPTO ~150.

stave_test::plugin_checks!(stave_multiband_comp_vst3::MultibandComp, {
    perf: vec![
        PerfCase { name: "DEFAULT", set: &[] },
        PerfCase { name: "DRIVEN", set: &[("lo drive", 12.0), ("mid drive", 12.0), ("hi drive", 12.0)] },
        PerfCase { name: "OPTO", set: &[("lo style", 2.0), ("mid style", 2.0), ("hi style", 2.0)] },
    ],
});
