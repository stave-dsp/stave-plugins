//! Every check Saga owes (validation, no allocation while every param
//! sweeps — MODE wipes the core mid-block, DIVE moves the clock ratio
//! so the bridge runs 0..n core ticks per frame — perf, the face's
//! laws, host behaviour), from one line.
//!
//! The perf cases: each program at full loops, plus the worst clock
//! case (program IV stretched fast AND risen 2× — the bridge runs the
//! most core ticks per host frame). Baselines 2026-10-25, Apple
//! Silicon, ns/frame @48k: I 238, II 216, III 226, IV 258, V 154,
//! IV fastest clock 430 (~1–2% of budget).

stave_test::plugin_checks!(stave_saga_vst3::Saga, {
    perf: vec![
        PerfCase { name: "I rev→rev", set: &[("mode", 0.0), ("feedback", 0.8), ("regen", 0.8)] },
        PerfCase { name: "II rev→oct-up", set: &[("mode", 1.0), ("feedback", 0.8), ("regen", 0.8)] },
        PerfCase { name: "III rev→oct-down", set: &[("mode", 2.0), ("feedback", 0.8), ("regen", 0.8)] },
        PerfCase { name: "IV rev-verb→verb", set: &[("mode", 3.0), ("feedback", 0.8), ("regen", 0.8)] },
        PerfCase { name: "V pitch→pitch", set: &[("mode", 4.0), ("feedback", 0.8), ("regen", 0.8)] },
        PerfCase {
            name: "IV fastest clock",
            set: &[("mode", 3.0), ("time", 50.0), ("dive", 1.0), ("shift", 2.0), ("ramp", 50.0)],
        },
    ],
});
