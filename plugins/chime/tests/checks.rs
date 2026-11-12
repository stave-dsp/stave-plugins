//! Every check Chime owes (validation, no allocation under the note
//! storm, perf, the face's laws, host behaviour), from one line. The
//! perf report holds an 8-note chord for instruments (an idle rack's
//! cost is not the number that matters). Baselines 2026-07-07 (Apple
//! Silicon, ns/frame): SINE CHORD ~49 (the f64 sin per voice), SAW
//! CHORD ~21 — both ≤ 0.2% of budget.

stave_test::plugin_checks!(stave_chime_vst3::Chime, {
    perf: vec![
        PerfCase { name: "SINE CHORD", set: &[] },
        PerfCase { name: "SAW CHORD", set: &[("shape", 1.0)] },
    ],
});
