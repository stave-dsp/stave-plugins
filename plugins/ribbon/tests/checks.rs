//! Every check Ribbon owes (validation, no allocation, perf, the face's
//! laws, host behaviour), from one line.

stave_test::plugin_checks!(stave_ribbon_vst3::Ribbon, {
    perf: vec![
        PerfCase { name: "DEFAULT", set: &[] },
        PerfCase {
            name: "worn + driven + runaway",
            set: &[("wear", 1.0), ("drive", 1.0), ("intensity", 1.0), ("spring", 1.0)],
        },
        PerfCase { name: "fast tape, erase off", set: &[("rate", 1.6), ("erase", 0.0)] },
    ],
    // Output trim stays in the host's menus.
    off_face: &["output"],
});
