//! Every check Tolex owes (validation, no allocation while every param
//! moves, perf, the face's laws, host behaviour), from one line. The
//! perf report documents, it does not block: record the measured
//! baselines HERE when they legitimately move.

stave_test::plugin_checks!(stave_tolex_vst3::Tolex, {
    perf: vec![
        PerfCase { name: "DEFAULT", set: &[] },
        PerfCase { name: "OPEN BACK", set: &[("back", 1.0)] },
        PerfCase { name: "TUBE + DRIVE", set: &[("damping", 1.0), ("drive", 24.0)] },
    ],
});
