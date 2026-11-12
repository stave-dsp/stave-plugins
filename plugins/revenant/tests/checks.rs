//! Every check Revenant owes (validation, no allocation, perf, the
//! face's laws, host behaviour), from one line.

stave_test::plugin_checks!(stave_revenant_vst3::Revenant, {
    perf: vec![
        PerfCase { name: "DEFAULT", set: &[] },
        PerfCase { name: "learning hard, long horizon", set: &[("learn", 1.0), ("horizon", 25.0), ("chaos", 0.8)] },
        PerfCase { name: "asleep, dreaming", set: &[("sleep", 1.0), ("dream", 1.0)] },
    ],
    // Output trim stays in the host's menus.
    off_face: &["output"],
});
