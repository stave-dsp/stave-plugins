//! Every check Aeolian owes (validation, no allocation, perf, the face's
//! laws, host behaviour), from one line.

stave_test::plugin_checks!(stave_aeolian_vst3::Aeolian, {
    perf: vec![
        PerfCase { name: "DEFAULT", set: &[] },
        PerfCase {
            name: "everything on",
            set: &[("sustain", 1.0), ("wind", 1.0), ("sway", 1.0), ("air", 1.0), ("drift", 1.0)],
        },
        PerfCase { name: "HIGH register", set: &[("register", 4.0), ("sustain", 0.9)] },
    ],
    // Output trim stays in the host's menus (see editor.rs).
    off_face: &["output"],
});
