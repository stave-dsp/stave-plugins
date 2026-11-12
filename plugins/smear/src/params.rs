//! Smear's parameter identity — ids are forever (1–12). The garden's
//! controls glide slowly (50 ms) so the field drifts rather than steps;
//! the excitation and tone controls at 20 ms, the gains at 5 ms.

use stave::prelude::*;

#[derive(Params)]
pub struct SmearParams {
    pub seed: FloatParam,
    pub smear: FloatParam,
    pub bloom: FloatParam,
    pub decay: FloatParam,
    pub flux: FloatParam,
    pub shimmer: FloatParam,
    pub drive: FloatParam,
    pub motion: FloatParam,
    pub depth: FloatParam,
    pub tone: FloatParam,
    pub mix: FloatParam,
    pub output: FloatParam,
}

impl Default for SmearParams {
    fn default() -> Self {
        let unit01 = |id: u32, name: &'static str, default: f64, group: &'static str, ms| {
            FloatParam::new(ParamId(id), name, Range::linear(0.0, 1.0))
                .default(default)
                .group(group)
                .smooth(Smooth::ms(ms))
        };
        Self {
            seed: unit01(1, "seed", 0.5, "Garden", 20.0),
            smear: unit01(2, "smear", 0.35, "Garden", 50.0),
            bloom: unit01(3, "bloom", 0.3, "Garden", 50.0),
            decay: unit01(4, "decay", 0.55, "Garden", 50.0),
            flux: unit01(5, "flux", 0.3, "Motion", 50.0),
            shimmer: unit01(6, "shimmer", 0.0, "Motion", 50.0),
            drive: unit01(7, "drive", 0.25, "Tone", 20.0),
            motion: unit01(8, "motion", 0.3, "Motion", 50.0),
            depth: unit01(9, "depth", 0.4, "Motion", 50.0),
            tone: unit01(10, "tone", 0.4, "Output", 20.0),
            mix: unit01(11, "mix", 0.5, "Output", 5.0),
            output: FloatParam::new(ParamId(12), "output", Range::linear(-24.0, 12.0))
                .default(0.0)
                .unit(Unit::Db)
                .group("Output")
                .smooth(Smooth::ms(5.0)),
        }
    }
}
