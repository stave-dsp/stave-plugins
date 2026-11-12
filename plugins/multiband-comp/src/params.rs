//! Multiband's parameter table.
//!
//! Ids are forever. Bands interleave in the original declaration order,
//! so `BandParams::new` derives each band's ids from per-field bases.
//! Retired ids, never reused: 3 (lm/hm), 4 (attack), 5 (release),
//! 48 (input gain), 49 (output gain), 51 (mix) — none reached the
//! signal; sessions that carry them keep them as unknown state.

use stave::prelude::*;

use crate::saturation::SaturationCharacter;
use crate::style::Style;

pub const BANDS: usize = 3;

/// The controls' glide: 20 ms, slow enough that threshold and crossover
/// sweeps never zipper.
const GLIDE: Smooth = Smooth::ms(20.0);

const DB: Unit = Unit::Db;

macro_rules! band_names {
    ($field:literal) => {
        [
            concat!("lo ", $field),
            concat!("mid ", $field),
            concat!("hi ", $field),
        ]
    };
}

const GROUPS: [&str; BANDS] = ["Low Band", "Mid Band", "High Band"];

#[derive(Params)]
pub struct BandParams {
    pub threshold: FloatParam,
    pub ratio: FloatParam,
    pub knee: FloatParam,
    pub makeup: FloatParam,
    pub attack: FloatParam,
    pub release: FloatParam,
    pub style: EnumParam<Style>,
    /// Transient catch: shortens attack on jumps; 0 leaves it to the style.
    pub transient: FloatParam,
    /// Peak → RMS blend of the detector input.
    pub rms: FloatParam,
    /// Detector high-pass; below 10 Hz it is off.
    pub sidechain_hpf: FloatParam,
    pub drive: FloatParam,
    pub saturation: EnumParam<SaturationCharacter>,
    pub saturation_mix: FloatParam,
    /// Compressed-band against dry-band blend.
    pub mix: FloatParam,
    pub solo: BoolParam,
    pub mute: BoolParam,
}

impl BandParams {
    fn new(band: usize) -> Self {
        let id = |base: u32, stride: u32| base + stride * band as u32;
        let group = GROUPS[band];
        let float = |id: u32, names: [&'static str; BANDS], default, range, unit| {
            FloatParam::new(ParamId(id), names[band], range)
                .default(default)
                .group(group)
                .unit(unit)
                .smooth(GLIDE)
        };
        let unit_range = Range::linear(0.0, 1.0);
        let plain = Unit::Plain { decimals: 2 };
        Self {
            threshold: float(
                id(6, 3),
                band_names!("threshold"),
                -18.0,
                Range::linear(-60.0, 0.0),
                DB,
            ),
            ratio: float(
                id(7, 3),
                band_names!("ratio"),
                3.0,
                Range::log(1.0, 20.0),
                Unit::Ratio,
            ),
            makeup: float(
                id(8, 3),
                band_names!("makeup"),
                0.0,
                Range::linear(-12.0, 24.0),
                DB,
            ),
            drive: float(
                id(15, 1),
                band_names!("drive"),
                0.0,
                Range::linear(0.0, 20.0),
                DB,
            ),
            saturation_mix: float(id(18, 1), band_names!("sat mix"), 1.0, unit_range, plain),
            style: EnumParam::new(ParamId(id(21, 1)), band_names!("style")[band], Style::Vca)
                .group(group),
            saturation: EnumParam::new(
                ParamId(id(24, 1)),
                band_names!("sat type")[band],
                SaturationCharacter::Tube,
            )
            .group(group),
            attack: float(
                id(27, 1),
                band_names!("attack"),
                8.0,
                Range::log(0.5, 200.0),
                Unit::Ms,
            ),
            release: float(
                id(30, 1),
                band_names!("release"),
                100.0,
                Range::log(5.0, 1000.0),
                Unit::Ms,
            ),
            mix: float(id(33, 1), band_names!("mix"), 1.0, unit_range, plain),
            sidechain_hpf: float(
                id(36, 1),
                band_names!("sc HPF"),
                0.0,
                Range::linear(0.0, 500.0),
                Unit::Hz,
            ),
            rms: float(id(39, 1), band_names!("rms mix"), 0.0, unit_range, plain),
            knee: float(
                id(42, 1),
                band_names!("knee"),
                0.0,
                Range::linear(0.0, 20.0),
                DB,
            ),
            transient: float(id(45, 1), band_names!("trans"), 0.0, unit_range, plain),
            solo: BoolParam::new(ParamId(id(52, 2)), band_names!("solo")[band], false).group(group),
            mute: BoolParam::new(ParamId(id(53, 2)), band_names!("mute")[band], false).group(group),
        }
    }
}

#[derive(Params)]
pub struct MbParams {
    pub xover_low: FloatParam,
    pub xover_high: FloatParam,
    #[param(group)]
    pub bands: [BandParams; BANDS],
    /// The framework gate owns click-free, latency-matched bypass.
    pub bypass: BoolParam,
    /// Detect from the sidechain input, when the host connects one.
    pub external_sidechain: BoolParam,
}

impl Default for MbParams {
    fn default() -> Self {
        Self {
            xover_low: FloatParam::new(ParamId(1), "low/mid", Range::log(60.0, 1000.0))
                .default(250.0)
                .group("Crossover")
                .unit(Unit::Hz)
                .smooth(GLIDE),
            xover_high: FloatParam::new(ParamId(2), "mid/high", Range::log(1000.0, 10000.0))
                .default(2500.0)
                .group("Crossover")
                .unit(Unit::Hz)
                .smooth(GLIDE),
            bands: std::array::from_fn(BandParams::new),
            bypass: BoolParam::bypass(ParamId(50), "bypass").group("Output"),
            external_sidechain: BoolParam::new(ParamId(58), "ext sc", false).group("Sidechain"),
        }
    }
}
