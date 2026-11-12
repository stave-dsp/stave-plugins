//! Every check Heirloom owes (validation, no allocation while every
//! param sweeps — MACHINE wipes the core mid-block, HOLD moves the
//! clock ratio so the bridge runs 0..n core ticks per frame, MEDIA /
//! WARP re-voice the wear — perf, the face's laws, host behaviour),
//! from one line.
//!
//! The perf cases: each machine at full loops with the record worn
//! hard, plus the worst clock case (BLOOM stretched fast while HOLD
//! sags it).

stave_test::plugin_checks!(stave_heirloom_vst3::Heirloom, {
    perf: vec![
        PerfCase { name: "REVERSE worn", set: &[("machine", 0.0), ("repeats", 0.8), ("age", 0.8), ("wobble", 0.6)] },
        PerfCase { name: "OCTAVE worn", set: &[("machine", 1.0), ("repeats", 0.8), ("age", 0.8), ("media", 1.0)] },
        PerfCase { name: "BLOOM worn", set: &[("machine", 2.0), ("repeats", 0.8), ("age", 0.8), ("media", 2.0)] },
        PerfCase { name: "BLOOM fast + held", set: &[("machine", 2.0), ("time", 50.0), ("hold", 1.0)] },
    ],
    off_face: &["output"],
});
