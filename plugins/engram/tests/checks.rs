//! Every check Engram owes (validation, no allocation while every param
//! moves, perf, the face's laws, host behaviour), from one line. The
//! perf cases: the defaults, the heaviest attention (big, long, soft
//! focus, every memory eligible), and HOLD (the room lossless).

stave_test::plugin_checks!(stave_engram_vst3::Engram, {
    perf: vec![
        PerfCase { name: "DEFAULT", set: &[] },
        PerfCase { name: "big, long, soft focus", set: &[("size", 1.0), ("decay", 30.0), ("focus", 0.0), ("deja", 0.0), ("recall", 1.0)] },
        PerfCase { name: "held", set: &[("hold", 1.0)] },
    ],
    off_face: &["output"],
});
