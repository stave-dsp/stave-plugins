//! Halo's factory presets.

use stave::prelude::*;

use crate::params::HaloParameters;

pub fn build_presets(p: &HaloParameters) -> Vec<Preset> {
    // (output, gain, bass, mid, treble, headroom, bloom, reserve, response)
    let preset = |name, v: [f64; 9]| {
        Preset::from_pairs(
            name,
            &[
                (p.output_level.id(), v[0]),
                (p.preamp_gain.id(), v[1]),
                (p.bass_tone.id(), v[2]),
                (p.midrange_tone.id(), v[3]),
                (p.treble_tone.id(), v[4]),
                (p.headroom.id(), v[5]),
                (p.bloom.id(), v[6]),
                (p.reserve.id(), v[7]),
                (p.response.id(), v[8]),
            ],
        )
    };
    vec![
        preset("OPEN", [0.0, 6.0, 5.5, 6.0, 5.5, 0.7, 0.45, 0.7, 0.6]),
        preset("GLASS", [0.0, 3.5, 4.5, 4.0, 7.5, 0.95, 0.2, 0.9, 0.9]),
        preset("BLOOM", [0.0, 9.0, 6.0, 7.0, 5.0, 0.55, 0.9, 0.65, 0.25]),
        preset("PIANO", [0.0, 5.0, 6.5, 6.5, 5.0, 0.85, 0.35, 1.0, 0.75]),
    ]
}
