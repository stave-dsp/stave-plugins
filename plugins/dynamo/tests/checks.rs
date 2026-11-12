//! Every check Dynamo owes (validation, no allocation under the note
//! storm, perf, the face's laws, host behaviour), from one line. For
//! an instrument the perf report holds an 8-note chord (an idle rack's
//! cost is not the number that matters). Baselines 2026-07-07 (Apple
//! Silicon, ns/frame): SINE CHORD ~49 (the f64 sin per voice), SAW
//! CHORD ~21 — both ≤ 0.2% of budget.

stave_test::plugin_checks!(stave_dynamo_vst3::Dynamo, {
    perf: vec![
        PerfCase { name: "SINE CHORD", set: &[] },
        PerfCase { name: "PCM WHEEL CHORD", set: &[("wheel", 7.0), ("poles", 6.0)] },
    ],
});
