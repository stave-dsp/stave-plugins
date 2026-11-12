//! Every check Forge owes (validation, no allocation, perf, the face's
//! laws, host behaviour), from one line. Perf baselines at adoption
//! (2026-07-06, Apple Silicon): ~670 ns/frame, 3.2% of budget — the
//! nested 4× oversampled M/S strip pair is the line's heaviest engine
//! by design.

stave_test::plugin_checks!(stave_forge_vst3::Forge, {
    perf: vec![
        PerfCase { name: "DEFAULT", set: &[] },
        PerfCase { name: "HOT", set: &[("drive", 9.0), ("depth", 0.8), ("sub mix", 0.6), ("grit", 0.7)] },
    ],
});
