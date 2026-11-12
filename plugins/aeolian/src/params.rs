//! Aeolian's typed parameter table — the face, in the harp's own
//! words. Ids are FOREVER — pick once, never reuse or renumber.
//!
//! ```text
//!   ( WIND )   ┃┃┃┃┃┃┃┃┃┃┃┃   ( SUSTAIN )     ← the harp window:
//!              C C# D … A# B                     twelve live strings
//!   tone  drift  REGISTER  ◉air ◉sway  mix
//!   aeolian                         ✦ [foot]
//! ```

use stave::prelude::*;

param_enum! {
    /// The strings' register, named by its lowest string: C2 puts the
    /// twelve strings in octave 2 (C2 … B2), each step lifts them an
    /// octave, C6 is octave 6.
    pub const REGISTERS: Register = ["C2", "C3", "C4", "C5", "C6"];
}

#[derive(Params)]
pub struct AeolianParams {
    /// Output trim (host-side; not on the face).
    pub output: FloatParam,
    /// The footswitch (inverted: the jewel lamp lights when engaged).
    /// The framework gate owns click-free bypass.
    pub bypass: BoolParam,
    /// Clean dry ↔ the strings (bit-exact dry at 0).
    pub mix: FloatParam,
    /// Dark ↔ bright: the strings' damping and the wet's top end.
    pub tone: FloatParam,
    /// Depth of the motion: AIR's spread and SWAY's drift.
    pub drift: FloatParam,
    /// WIND: how easily a played note wakes its string — velocity. Near the
    /// top (with SUSTAIN near the top too) the noise floor alone is
    /// enough: the strings self-oscillate.
    pub wind: FloatParam,
    /// SUSTAIN: how long a woken string rings — the envelope's decay.
    pub sustain: FloatParam,
    /// Which octave the strings are tuned in (the chicken-head selector).
    pub register: EnumParam<Register>,
    /// AIR: an ensemble of three drifting copies — width.
    pub air: BoolParam,
    /// SWAY: each string's pitch drifts on its own.
    pub sway: BoolParam,
}

impl Default for AeolianParams {
    fn default() -> Self {
        let unit = || Range::linear(0.0, 1.0);
        Self {
            output: FloatParam::new(ParamId(1), "output", Range::linear(-24.0, 24.0))
                .default(0.0)
                .unit(Unit::Db)
                .group("Output")
                .smooth(Smooth::ms(5.0)),
            bypass: BoolParam::bypass(ParamId(2), "bypass").group("Output"),
            mix: FloatParam::new(ParamId(3), "mix", unit())
                .default(0.5)
                .unit(Unit::Percent)
                .group("Strings")
                .smooth(Smooth::ms(10.0)),
            tone: FloatParam::new(ParamId(4), "tone", unit())
                .default(0.6)
                .unit(Unit::Percent)
                .group("Strings")
                .smooth(Smooth::ms(30.0)),
            drift: FloatParam::new(ParamId(5), "drift", unit())
                .default(0.3)
                .unit(Unit::Percent)
                .group("Motion")
                .smooth(Smooth::ms(40.0)),
            wind: FloatParam::new(ParamId(6), "wind", unit())
                .default(0.5)
                .unit(Unit::Percent)
                .group("Strings")
                .smooth(Smooth::ms(30.0)),
            sustain: FloatParam::new(ParamId(7), "sustain", unit())
                .default(0.6)
                .unit(Unit::Percent)
                .group("Strings")
                .smooth(Smooth::ms(50.0)),
            register: EnumParam::new(ParamId(8), "register", Register(2)).group("Strings"),
            air: BoolParam::new(ParamId(9), "air", true)
                .group("Motion")
                .names("OFF", "ON"),
            sway: BoolParam::new(ParamId(10), "sway", false)
                .group("Motion")
                .names("OFF", "ON"),
        }
    }
}
