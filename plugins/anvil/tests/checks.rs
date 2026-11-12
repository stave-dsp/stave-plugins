//! Every check Anvil owes (validation, no allocation while every param
//! moves, perf, the face's laws, host behaviour), from one line. The
//! perf cases are the representative configurations, measured against
//! the realtime budget (it documents, it does not block): record the
//! baselines HERE when they legitimately move.

stave_test::plugin_checks!(stave_anvil_vst3::Anvil, {
    perf: vec![
        PerfCase { name: "DEFAULT", set: &[] },
        PerfCase { name: "FULL SQUASH", set: &[("gain", 22.0), ("squash", 16.0), ("focus", 1.0)] },
    ],
});
