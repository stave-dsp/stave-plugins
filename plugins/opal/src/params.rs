//! Opal's typed parameter table. Ids are FOREVER — pick once, never
//! reuse or renumber. Four operators × (ratio, level, ADSR) + the
//! global surface. Per-operator ids are blocked in tens (op1 = 10s,
//! op2 = 20s, …) so the table reads like the panel.

use stave::prelude::*;
use stave::route::RouteParams;

param_enum! {
    /// The eight algorithms — mod wiring as data (engine/operators.rs
    /// owns the actual tables; these labels are the selector truth).
    /// CHAIN 4→3→2→1; FORK (3,4)→2→1; TWIN (2, 3←4)→1;
    /// STACK2 two 2-op pairs; TRIPLE 4→(1,2,3); BRIGHT (2,3,4)→1;
    /// ORGAN2 2→1 + two pure; ORGAN additive.
    pub const ALGOS: Algo = [
        "CHAIN", "FORK", "TWIN", "STACK2", "TRIPLE", "BRIGHT", "ORGAN2", "ORGAN",
    ];
}

/// The operator page labels — pure editor chrome (the pager is a
/// face Pager over editor-local page state: no param, no automation,
/// no state).
pub const OPS: &[&str] = &["OP 1", "OP 2", "OP 3", "OP 4"];

param_enum! {
    /// Modulation sources (the chime vocabulary). OFF
    /// disables. Per-voice: VEL, KEY, ENV (the carrier envelope),
    /// SPREAD (stable per-note random). Global: LFO, HEAT (fatigue
    /// integrator), SYMP (the other voices' energy).
    pub const SOURCES: ModSource =
        ["OFF", "LFO 1", "VEL", "KEY", "ENV", "SPREAD", "HEAT", "SYMP", "LFO 2"];
}

param_enum! {
    /// The LFO's waveform (dsp basis + RandomWalk recipes).
    pub const LFO_SHAPES: LfoShapeP = ["SINE", "TRI", "SAW", "SQR", "S&H", "DRIFT"];
}

param_enum! {
    /// Rate mode: free Hz, or tempo-locked divisions.
    pub const LFO_SYNC: LfoSync =
        ["FREE", "1/1", "1/2", "1/4", "1/8", "1/16", "1/4T", "1/8T"];
}

param_enum! {
    /// FREE = one global phase; NOTE = per-voice restart at note-on.
    pub const LFO_TRIG: LfoTrig = ["FREE", "NOTE"];
}

param_enum! {
    /// Modulation destinations (APPEND-only — step indices are
    /// state) — the DECLARED allowlist. The
    /// modulator LEVELS are the FM-musical ones: LFO→LEVEL 2 is the
    /// classic wah, KEY→LEVEL 4 is keyboard-tracked brightness.
    pub const TARGETS: ModTarget = [
        "PITCH", "LEVEL 2", "LEVEL 3", "LEVEL 4", "FEEDBACK",
        // The Operator lesson: op VOLUMES and pitch are first-class
        // destinations; RATIO 2 is inharmonicity (FM chorus at
        // small depths); LFO2 RATE is Absynth's cross-LFO motion.
        "LEVEL 1", "RATIO 2", "LFO2 RATE",
    ];
}

#[derive(Params)]
pub struct OpalParams {
    /// Output trim. Smoothing is DECLARED here (table-owned ramp).
    pub output: FloatParam,
    /// The framework gate owns click-free bypass; this carries the flag.
    pub bypass: BoolParam,
    /// Which wiring runs (mod matrix as data).
    pub algo: EnumParam<Algo>,
    /// Op 4's self-feedback — a pure sine grows sawtooth teeth.
    pub feedback: FloatParam,
    /// Velocity → modulator depth: 0 = velocity only moves loudness,
    /// 1 = soft notes are pure and hard notes are bright (the FM
    /// expressiveness knob).
    /// (id 6 was `op`, the editor's page selector masquerading as a
    /// param — replaced by a ViewKey; the id is never reused.)
    pub vel_mod: FloatParam,
    /// The matrix clock.
    pub lfo_rate: FloatParam,
    pub lfo_shape: EnumParam<LfoShapeP>,
    pub lfo_sync: EnumParam<LfoSync>,
    pub lfo_rise: FloatParam,
    pub lfo_trig: EnumParam<LfoTrig>,
    /// The SECOND LFO (the Absynth lesson: independent motions
    /// compound; one LFO is a wobble, two are weather). Same voice
    /// of options as LFO 1, defaulting slower.
    pub lfo2_rate: FloatParam,
    pub lfo2_shape: EnumParam<LfoShapeP>,
    pub lfo2_sync: EnumParam<LfoSync>,
    pub lfo2_rise: FloatParam,
    pub lfo2_trig: EnumParam<LfoTrig>,
    /// The modulation routes: seven ordinary params per slot (see
    /// `RouteParams`), so automation, state and presets cover them.
    #[param(group)]
    pub routes: [RouteParams<ModSource, ModTarget>; 4],
    // ---- OP 1 (the anchor carrier) ------------------------------
    pub ratio1: FloatParam,
    pub level1: FloatParam,
    pub attack1: FloatParam,
    pub decay1: FloatParam,
    pub sustain1: FloatParam,
    pub release1: FloatParam,
    // ---- OP 2 ----------------------------------------------------
    pub ratio2: FloatParam,
    pub level2: FloatParam,
    pub attack2: FloatParam,
    pub decay2: FloatParam,
    pub sustain2: FloatParam,
    pub release2: FloatParam,
    // ---- OP 3 ----------------------------------------------------
    pub ratio3: FloatParam,
    pub level3: FloatParam,
    pub attack3: FloatParam,
    pub decay3: FloatParam,
    pub sustain3: FloatParam,
    pub release3: FloatParam,
    // ---- OP 4 (the top of every stack; owns the feedback) --------
    pub ratio4: FloatParam,
    pub level4: FloatParam,
    pub attack4: FloatParam,
    pub decay4: FloatParam,
    pub sustain4: FloatParam,
    pub release4: FloatParam,
}

fn ratio(id: u32, name: &'static str) -> FloatParam {
    FloatParam::new(ParamId(id), name, Range::log(0.25, 16.0))
        .default(1.0)
        .group("Operators")
        .smooth(Smooth::Off)
}

fn level(id: u32, name: &'static str, default: f64) -> FloatParam {
    FloatParam::new(ParamId(id), name, Range::linear(0.0, 1.0))
        .default(default)
        .group("Operators")
        .smooth(Smooth::Off)
}

fn a(id: u32, name: &'static str, default: f64) -> FloatParam {
    FloatParam::new(ParamId(id), name, Range::log(0.5, 2_000.0))
        .default(default)
        .unit(Unit::Ms)
        .group("Operators")
        .smooth(Smooth::Off)
}

fn d(id: u32, name: &'static str, default: f64) -> FloatParam {
    FloatParam::new(ParamId(id), name, Range::log(1.0, 4_000.0))
        .default(default)
        .unit(Unit::Ms)
        .group("Operators")
        .smooth(Smooth::Off)
}

fn sus(id: u32, name: &'static str, default: f64) -> FloatParam {
    FloatParam::new(ParamId(id), name, Range::linear(0.0, 1.0))
        .default(default)
        .group("Operators")
        .smooth(Smooth::Off)
}

fn r(id: u32, name: &'static str, default: f64) -> FloatParam {
    FloatParam::new(ParamId(id), name, Range::log(2.0, 8_000.0))
        .default(default)
        .unit(Unit::Ms)
        .group("Operators")
        .smooth(Smooth::Off)
}

impl Default for OpalParams {
    fn default() -> Self {
        Self {
            output: FloatParam::new(ParamId(1), "output", Range::linear(-24.0, 24.0))
                .unit(Unit::Db)
                .group("Output")
                .smooth(Smooth::ms(5.0)),
            bypass: BoolParam::bypass(ParamId(2), "bypass").group("Output"),
            algo: EnumParam::new(ParamId(3), "algo", Algo(0)).group("Global"),
            feedback: FloatParam::new(ParamId(4), "feedback", Range::linear(0.0, 1.0))
                .group("Global")
                .smooth(Smooth::Off),
            vel_mod: FloatParam::new(ParamId(5), "vel mod", Range::linear(0.0, 1.0))
                .default(0.5)
                .group("Global")
                .smooth(Smooth::Off),
            lfo_shape: EnumParam::new(ParamId(47), "lfo shape", LfoShapeP(0)).group("Mod"),
            lfo_sync: EnumParam::new(ParamId(48), "lfo sync", LfoSync(0)).group("Mod"),
            lfo_rise: FloatParam::new(ParamId(49), "lfo rise", Range::linear(0.0, 2_000.0))
                .unit(Unit::Ms)
                .group("Mod")
                .smooth(Smooth::Off),
            lfo_trig: EnumParam::new(ParamId(50), "lfo trig", LfoTrig(0)).group("Mod"),
            lfo2_rate: FloatParam::new(ParamId(88), "lfo2 rate", Range::log(0.05, 12.0))
                .default(0.3)
                .unit(Unit::Hz)
                .group("Mod")
                .smooth(Smooth::Off),
            lfo2_shape: EnumParam::new(ParamId(89), "lfo2 shape", LfoShapeP(0)).group("Mod"),
            lfo2_sync: EnumParam::new(ParamId(90), "lfo2 sync", LfoSync(0)).group("Mod"),
            lfo2_rise: FloatParam::new(ParamId(91), "lfo2 rise", Range::linear(0.0, 2_000.0))
            .unit(Unit::Ms)
            .group("Mod")
            .smooth(Smooth::Off),
            lfo2_trig: EnumParam::new(ParamId(92), "lfo2 trig", LfoTrig(0)).group("Mod"),
            lfo_rate: FloatParam::new(ParamId(7), "lfo rate", Range::log(0.05, 12.0))
                .default(1.2)
                .unit(Unit::Hz)
                .group("Mod")
                .smooth(Smooth::Off),
            routes: core::array::from_fn(|i| RouteParams::new(ParamId(50 + 10 * i as u32), i + 1, "Mod")),
            ratio1: ratio(10, "ratio 1"),
            level1: level(11, "level 1", 1.0),
            attack1: a(12, "attack 1", 2.0),
            decay1: d(13, "decay 1", 600.0),
            sustain1: sus(14, "sustain 1", 0.7),
            release1: r(15, "release 1", 300.0),

            ratio2: ratio(20, "ratio 2"),
            level2: level(21, "level 2", 0.5),
            attack2: a(22, "attack 2", 2.0),
            decay2: d(23, "decay 2", 400.0),
            sustain2: sus(24, "sustain 2", 0.5),
            release2: r(25, "release 2", 300.0),

            ratio3: ratio(30, "ratio 3"),
            level3: level(31, "level 3", 0.0),
            attack3: a(32, "attack 3", 2.0),
            decay3: d(33, "decay 3", 400.0),
            sustain3: sus(34, "sustain 3", 0.5),
            release3: r(35, "release 3", 300.0),

            ratio4: ratio(40, "ratio 4"),
            level4: level(41, "level 4", 0.0),
            attack4: a(42, "attack 4", 2.0),
            decay4: d(43, "decay 4", 400.0),
            sustain4: sus(44, "sustain 4", 0.5),
            release4: r(45, "release 4", 300.0),
        }
    }
}
