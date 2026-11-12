//! Every check Smear owes (validation, no allocation, perf, the face's
//! laws, host behaviour), from one line. Perf baseline 2026-10-13
//! (Apple Silicon, ns/frame): DEFAULT ~810, FULL BLOOM ~950.

stave_test::plugin_checks!(stave_smear_vst3::Smear, {
    perf: vec![
        PerfCase { name: "DEFAULT", set: &[] },
        PerfCase {
            name: "FULL BLOOM",
            set: &[("smear", 1.0), ("bloom", 1.0), ("flux", 1.0), ("shimmer", 1.0)],
        },
    ],
});
