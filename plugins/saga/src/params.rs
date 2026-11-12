//! Saga's typed parameter table. Ids are FOREVER — pick once,
//! never reuse or renumber. The table reads like the pedal it
//! answers (docs/saga.md): the eight face knobs, the MODE button,
//! the five TWEAK functions broken out as their own knobs, the
//! Tap/Shift footswitch's hold as DIVE, and host sync where the
//! pedal has tap tempo.

use stave::prelude::*;

param_enum! {
    /// The five programs — each pairs two machines in series, each
    /// with its own feedback path (FEEDBACK drives the first, REGEN
    /// the second):
    /// I   reverse delay → reverse reverb
    /// II  reverse delay → octave-up reverb
    /// III reverse delay → octave-down reverb
    /// IV  reverse reverb → forward reverb
    /// V   pitch delay → pitch delay
    pub const MODES: Mode = ["I", "II", "III", "IV", "V"];
}

param_enum! {
    /// The modulation LFO's shapes (TWEAK → MOD SHAPE).
    pub const MOD_SHAPES: ModShape = ["SINE", "SQUARE", "RANDOM"];
}

param_enum! {
    /// TIME's clock source. FREE = the knob; the divisions follow the
    /// host tempo (the plugin's answer to the pedal's tap switch).
    pub const SYNCS: Sync = ["FREE", "1/2", "1/4", "1/8.", "1/8", "1/8T", "1/16"];
}

#[derive(Params)]
pub struct SagaParams {
    /// Output trim. Smoothing is DECLARED here (table-owned ramp).
    pub output: FloatParam,
    /// The host's bypass switch; the framework runs it (click-free).
    pub bypass: BoolParam,
    /// Which program runs.
    pub mode: EnumParam<Mode>,
    /// Gain of the FIRST feedback path — delay repeats (I–III, V), or
    /// octave-down + reverse-reverb feedback (IV).
    pub feedback: FloatParam,
    /// Gain of the SECOND feedback path — reverse-reverb feedback (I),
    /// octave repeats into the tank (II, III), octave-up + reverse
    /// feedback (IV), the second pitch delay's repeats (V).
    pub regen: FloatParam,
    /// Depth of the clock modulation — the whole wet machine wobbles.
    pub mod_depth: FloatParam,
    /// Wet/dry: fully left = dry (bit-exact, a property test), right = wet.
    pub mix: FloatParam,
    /// The mode-dependent knob: how reversed (I), reverb decay
    /// (II–IV), interval order of the pitch sequence (V).
    pub x: FloatParam,
    /// Delay time (I–III, V); the time-stretch of the space (IV).
    pub time: FloatParam,
    /// The wet path's synth-style low-pass: closed = lows only, open
    /// = the full band.
    pub tone: FloatParam,
    /// TWEAK: how far the wet image spreads between the speakers.
    pub width: FloatParam,
    /// TWEAK: the modulation LFO's rate.
    pub mod_rate: FloatParam,
    /// TWEAK: the modulation LFO's shape.
    pub mod_shape: EnumParam<ModShape>,
    /// TWEAK: where DIVE/RISE takes the clock — 0.5× dives an octave,
    /// 2× rises one.
    pub shift: FloatParam,
    /// TWEAK: how long the clock takes to get to SHIFT, and back.
    pub ramp: FloatParam,
    /// The Tap/Shift footswitch held down: the clock glides to SHIFT
    /// and stays there while this is on.
    pub dive: BoolParam,
    /// TIME follows the host tempo at this division (FREE = the knob).
    pub sync: EnumParam<Sync>,
}

impl Default for SagaParams {
    fn default() -> Self {
        let unit = || Range::linear(0.0, 1.0);
        Self {
            output: FloatParam::new(ParamId(1), "output", Range::linear(-24.0, 24.0))
                .unit(Unit::Db)
                .group("Output")
                .smooth(Smooth::ms(5.0)),
            bypass: BoolParam::bypass(ParamId(2), "bypass").group("Output"),
            mode: EnumParam::new(ParamId(3), "mode", Mode(0)).group("Program"),
            feedback: FloatParam::new(ParamId(4), "feedback", unit())
                .default(0.45)
                .unit(Unit::Percent)
                .group("Program")
                .smooth(Smooth::ms(30.0)),
            regen: FloatParam::new(ParamId(5), "regen", unit())
                .default(0.4)
                .unit(Unit::Percent)
                .group("Program")
                .smooth(Smooth::ms(30.0)),
            mod_depth: FloatParam::new(ParamId(6), "mod", unit())
                .default(0.25)
                .unit(Unit::Percent)
                .group("Program")
                .smooth(Smooth::ms(50.0)),
            mix: FloatParam::new(ParamId(7), "mix", unit())
                .default(0.5)
                .unit(Unit::Percent)
                .group("Program")
                .smooth(Smooth::ms(10.0)),
            x: FloatParam::new(ParamId(8), "x", unit())
                .default(0.5)
                .unit(Unit::Percent)
                .group("Program")
                .smooth(Smooth::ms(60.0)),
            time: FloatParam::new(ParamId(9), "time", Range::log(50.0, 1200.0))
                .default(450.0)
                .unit(Unit::Ms)
                .group("Program")
                .smooth(Smooth::ms(150.0)),
            tone: FloatParam::new(ParamId(10), "tone", unit())
                .default(0.75)
                .unit(Unit::Percent)
                .group("Program")
                .smooth(Smooth::ms(20.0)),
            width: FloatParam::new(ParamId(11), "width", unit())
                .default(0.6)
                .unit(Unit::Percent)
                .group("Tweak")
                .smooth(Smooth::ms(20.0)),
            mod_rate: FloatParam::new(ParamId(12), "mod rate", Range::log(0.1, 5.0))
                .default(0.6)
                .unit(Unit::Hz)
                .group("Tweak")
                .smooth(Smooth::ms(50.0)),
            mod_shape: EnumParam::new(ParamId(13), "mod shape", ModShape(0)).group("Tweak"),
            shift: FloatParam::new(ParamId(14), "shift", Range::log(0.5, 2.0))
                .default(0.5)
                .unit(Unit::Plain { decimals: 2 })
                .group("Tweak"),
            ramp: FloatParam::new(ParamId(15), "ramp", Range::log(50.0, 5000.0))
                .default(600.0)
                .unit(Unit::Ms)
                .group("Tweak"),
            dive: BoolParam::new(ParamId(16), "dive", false).group("Tweak").names("OFF", "HELD"),
            sync: EnumParam::new(ParamId(17), "sync", Sync(0)).group("Program"),
        }
    }
}
