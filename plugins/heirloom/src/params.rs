//! Heirloom's typed parameter table — the pedal's face and nothing
//! else. Ids are FOREVER — pick once, never reuse or renumber.
//! Six knobs, three toggles, two footswitches (docs/design.md):
//!
//! ```text
//!   TIME     REPEATS   MIX        ← the saga core (reverse machines)
//!  [MACHINE] [ WARP ]  [MEDIA]    ← three-way toggles
//!   WOBBLE   RPM       AGE        ← the media stage (vinyl + tape)
//!   (bypass)           (hold)     ← footswitches
//! ```

use stave::prelude::*;

param_enum! {
    /// Which reverse machine runs the wet core (saga programs I, II, IV):
    /// REVERSE  reverse delay → reverse reverb
    /// OCTAVE   reverse delay → octave-up reverb
    /// BLOOM    reverse reverb → forward reverb (TIME stretches the space)
    pub const MACHINES: Machine = ["REVERSE", "OCTAVE", "BLOOM"];
}

param_enum! {
    /// The wobble's shape — the record's warp:
    /// SINE   an even, seasick bend
    /// WARP   a warped platter: a fast lurch up, a long sag down
    /// DRIFT  no period at all — smoothed random wandering
    pub const WARPS: Warp = ["SINE", "WARP", "DRIFT"];
}

param_enum! {
    /// What AGE wears down — each medium decays its own way:
    /// VINYL  crackle, rumble, the 33⅓ off-centre wow, gentle top loss
    /// TAPE   hiss, saturation, flutter, dropouts
    /// VHS    narrow band, heavy hiss, jittery flutter, deep dropouts
    pub const MEDIAS: Media = ["VINYL", "TAPE", "VHS"];
}

param_enum! {
    /// TIME's clock source. FREE = the knob; the divisions follow the
    /// host tempo (the pedal's DIP-switch sync, as a host param).
    pub const SYNCS: Sync = ["FREE", "1/2", "1/4", "1/8.", "1/8", "1/8T", "1/16"];
}

#[derive(Params)]
pub struct HeirloomParams {
    /// Output trim (host-side; not on the face).
    pub output: FloatParam,
    /// Left footswitch: the host's bypass switch; the framework runs it
    /// (click-free).
    pub bypass: BoolParam,
    /// Delay time (REVERSE, OCTAVE); the time-stretch of the space (BLOOM).
    pub time: FloatParam,
    /// Both feedback paths together — repeats, and how long the
    /// reversed space holds. Near the top the loops sustain.
    pub repeats: FloatParam,
    /// Clean dry ↔ the whole pedal (dry + reverse machine, played off
    /// the worn record). WOBBLE / RPM / AGE live on the wet side, so
    /// MIX 0 is untouched dry and 50% is the Warped Vinyl chorus.
    pub mix: FloatParam,
    /// Depth of the platter wobble (Warped Vinyl's DEPTH).
    pub wobble: FloatParam,
    /// The wobble's speed (Warped Vinyl's RPM).
    pub rpm: FloatParam,
    /// How worn the medium is (Generation Loss's whole face, as one
    /// knob): bandwidth, saturation, noise floor, flutter, failures.
    pub age: FloatParam,
    pub machine: EnumParam<Machine>,
    pub warp: EnumParam<Warp>,
    pub media: EnumParam<Media>,
    /// Right footswitch: the wet clock sags an octave (a tape stop on
    /// the reverse machine) and stays there while latched.
    pub hold: BoolParam,
    /// TIME follows the host tempo at this division (FREE = the knob).
    pub sync: EnumParam<Sync>,
}

impl Default for HeirloomParams {
    fn default() -> Self {
        let unit = || Range::linear(0.0, 1.0);
        Self {
            output: FloatParam::new(ParamId(1), "output", Range::linear(-24.0, 24.0))
                .unit(Unit::Db)
                .group("Output")
                .smooth(Smooth::ms(5.0)),
            bypass: BoolParam::bypass(ParamId(2)).group("Output"),
            time: FloatParam::new(ParamId(3), "time", Range::log(50.0, 1200.0))
                .default(450.0)
                .unit(Unit::Ms)
                .group("Machine")
                .smooth(Smooth::ms(150.0)),
            repeats: FloatParam::new(ParamId(4), "repeats", unit())
                .default(0.5)
                .unit(Unit::Percent)
                .group("Machine")
                .smooth(Smooth::ms(30.0)),
            mix: FloatParam::new(ParamId(5), "mix", unit())
                .default(0.45)
                .unit(Unit::Percent)
                .group("Machine")
                .smooth(Smooth::ms(10.0)),
            wobble: FloatParam::new(ParamId(6), "wobble", unit())
                .default(0.3)
                .unit(Unit::Percent)
                .group("Media")
                .smooth(Smooth::ms(40.0)),
            rpm: FloatParam::new(ParamId(7), "rpm", Range::log(0.1, 8.0))
                .default(0.8)
                .unit(Unit::Hz)
                .group("Media")
                .smooth(Smooth::ms(60.0)),
            age: FloatParam::new(ParamId(8), "age", unit())
                .default(0.0)
                .unit(Unit::Percent)
                .group("Media")
                .smooth(Smooth::ms(40.0)),
            machine: EnumParam::new(ParamId(9), "machine", Machine(0)).group("Machine"),
            warp: EnumParam::new(ParamId(10), "warp", Warp(0)).group("Media"),
            media: EnumParam::new(ParamId(11), "media", Media(0)).group("Media"),
            hold: BoolParam::new(ParamId(12), "hold", false).group("Machine").names("OFF", "HELD"),
            sync: EnumParam::new(ParamId(13), "sync", Sync(0)).group("Machine"),
        }
    }
}
