//! Halo's stable host parameter table. Ids are FOREVER.

use stave::prelude::*;

pub const DEFAULT_PREAMP_GAIN_DECIBELS: f64 = 6.0;

#[derive(Params)]
pub struct HaloParameters {
    pub preamp_gain: FloatParam,
    pub bass_tone: FloatParam,
    pub midrange_tone: FloatParam,
    pub treble_tone: FloatParam,
    pub headroom: FloatParam,
    pub bloom: FloatParam,
    pub reserve: FloatParam,
    pub response: FloatParam,
    pub output_level: FloatParam,
    pub bypass: BoolParam,
}

impl Default for HaloParameters {
    fn default() -> Self {
        let tone = |id, name, default| {
            FloatParam::new(ParamId(id), name, Range::linear(0.0, 10.0))
                .default(default)
                .unit(Unit::Plain { decimals: 1 })
                .group("Tone")
        };
        let dynamics = |id, name, default| {
            FloatParam::new(ParamId(id), name, Range::linear(0.0, 1.0))
                .default(default)
                .group("Dynamics")
        };
        Self {
            output_level: FloatParam::new(ParamId(1), "output", Range::linear(-24.0, 24.0))
                .unit(Unit::Db)
                .group("Output")
                .smooth(Smooth::ms(5.0)),
            bypass: BoolParam::bypass(ParamId(2)).group("Output"),
            preamp_gain: FloatParam::new(ParamId(3), "gain", Range::linear(0.0, 18.0))
                .default(DEFAULT_PREAMP_GAIN_DECIBELS)
                .unit(Unit::Db)
                .group("Preamp")
                .smooth(Smooth::ms(8.0)),
            bass_tone: tone(4, "bass", 5.5),
            midrange_tone: tone(5, "mid", 6.0),
            treble_tone: tone(6, "treble", 5.5),
            headroom: dynamics(7, "headroom", 0.7),
            bloom: dynamics(8, "bloom", 0.45),
            reserve: dynamics(9, "reserve", 0.7),
            response: dynamics(10, "response", 0.6),
        }
    }
}
