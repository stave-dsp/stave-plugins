//! Ribbon's typed parameter table — the face. Ids are FOREVER — pick
//! once, never reuse or renumber.
//!
//! ```text
//!   ┌───────────── the tape window: the loop, the heads ─────────────┐
//!   RATE  INTENSITY  DRIVE  [HEADS]  WEAR  ERASE  SPRING  MIX
//!   (bypass)                 ribbon                        (brake)
//! ```

use stave::prelude::*;

param_enum! {
    /// Which playback heads are live. The heads sit 1 : 2 : 3 down the
    /// tape from the record head.
    pub const HEADS: Heads = ["1", "2", "3", "1+2", "2+3", "1+3", "1+2+3"];
}

#[derive(Params)]
pub struct RibbonParams {
    /// Output trim (host-side; not on the face).
    pub output: FloatParam,
    /// Left footswitch. The framework gate owns click-free bypass.
    pub bypass: BoolParam,
    /// REPEAT RATE — the capstan motor's speed (1 = nominal: heads at
    /// 90 / 180 / 270 ms). The motor has inertia, so turning it bends
    /// the pitch of everything on the tape.
    pub rate: FloatParam,
    /// How much of the heads goes back to the record head. Past ~85%
    /// the loop runs away into self-oscillation.
    pub intensity: FloatParam,
    /// The tube preamp into the record head (12AX7, Koren model).
    pub drive: FloatParam,
    pub heads: EnumParam<Heads>,
    /// The tape's age: wow and flutter, hiss, head-gap loss, dropouts,
    /// and each generation on the loop duller than the last.
    pub wear: FloatParam,
    /// The erase head. 100% wipes the tape before every recording (a
    /// plain echo); lower, the loop keeps what it held and the new pass
    /// records on top (sound-on-sound, a generation older each turn).
    pub erase: FloatParam,
    /// The spring tank's level.
    pub spring: FloatParam,
    /// Clean dry ↔ echoes + spring (bit-exact dry at 0).
    pub mix: FloatParam,
    /// Right footswitch: the motor winds down to a stop and stays there;
    /// release and it winds back up. The loop keeps its recording.
    pub brake: BoolParam,
}

impl Default for RibbonParams {
    fn default() -> Self {
        let unit = || Range::linear(0.0, 1.0);
        Self {
            output: FloatParam::new(ParamId(1), "output", Range::linear(-24.0, 24.0))
                .default(0.0)
                .unit(Unit::Db)
                .group("Output")
                .smooth(Smooth::ms(5.0)),
            bypass: BoolParam::bypass(ParamId(2), "bypass").group("Output"),
            rate: FloatParam::new(ParamId(3), "rate", Range::log(0.35, 1.6))
                .default(1.0)
                .unit(Unit::Plain { decimals: 2 })
                .group("Tape"),
            intensity: FloatParam::new(ParamId(4), "intensity", unit())
                .default(0.4)
                .unit(Unit::Percent)
                .group("Tape")
                .smooth(Smooth::ms(30.0)),
            drive: FloatParam::new(ParamId(5), "drive", unit())
                .default(0.25)
                .unit(Unit::Percent)
                .group("Tape")
                .smooth(Smooth::ms(30.0)),
            heads: EnumParam::new(ParamId(6), "heads", Heads(6)).group("Tape"),
            wear: FloatParam::new(ParamId(7), "wear", unit())
                .default(0.0)
                .unit(Unit::Percent)
                .group("Tape")
                .smooth(Smooth::ms(40.0)),
            erase: FloatParam::new(ParamId(8), "erase", unit())
                .default(1.0)
                .unit(Unit::Percent)
                .group("Tape")
                .smooth(Smooth::ms(40.0)),
            spring: FloatParam::new(ParamId(9), "spring", unit())
                .default(0.3)
                .unit(Unit::Percent)
                .group("Spring")
                .smooth(Smooth::ms(20.0)),
            mix: FloatParam::new(ParamId(10), "mix", unit())
                .default(0.4)
                .unit(Unit::Percent)
                .group("Output")
                .smooth(Smooth::ms(10.0)),
            brake: BoolParam::new(ParamId(11), "brake", false)
                .group("Tape")
                .names("RUN", "BRAKE"),
        }
    }
}
