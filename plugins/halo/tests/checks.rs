//! Every check Halo owes (validation, no allocation, perf, the face's
//! laws, host behaviour), from one line.

stave_test::plugin_checks!(stave_halo_vst3::Halo, {
    perf: vec![
        PerfCase { name: "OPEN", set: &[] },
        PerfCase { name: "MAXIMUM BLOOM", set: &[("bloom", 1.0), ("reserve", 1.0), ("response", 0.0)] },
    ],
});
