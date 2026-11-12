//! Factory presets — each states the FULL face.

use stave::prelude::*;

use crate::params::AeolianParams;

struct Face {
    mix: f64,
    tone: f64,
    drift: f64,
    wind: f64,
    sustain: f64,
    register: f64,
    air: f64,
    sway: f64,
}

fn face(name: &'static str, p: &AeolianParams, f: Face) -> Preset {
    Preset::from_pairs(
        name,
        &[
            (p.mix.id(), f.mix), (p.tone.id(), f.tone), (p.drift.id(), f.drift), (p.wind.id(), f.wind),
            (p.sustain.id(), f.sustain), (p.register.id(), f.register), (p.air.id(), f.air),
            (p.sway.id(), f.sway), (p.output.id(), 0.0),
        ],
    )
}

pub fn presets(p: &AeolianParams) -> Vec<Preset> {
    vec![
        face("INIT", p, Face {
            mix: 0.5, tone: 0.6, drift: 0.3, wind: 0.5, sustain: 0.6, register: 2.0, air: 1.0, sway: 0.0,
        }),
        // Long glacial swells: everything you play blooms and hangs.
        face("GLACIER", p, Face {
            mix: 0.6, tone: 0.45, drift: 0.5, wind: 0.6, sustain: 0.92, register: 2.0, air: 1.0, sway: 1.0,
        }),
        // Short bright plucks an octave up — a harp answering.
        face("HARP ECHO", p, Face {
            mix: 0.45, tone: 0.8, drift: 0.15, wind: 0.5, sustain: 0.35, register: 3.0, air: 0.0, sway: 0.0,
        }),
        // Low drone strings under the guitar: dark, wide, slow.
        face("CELLO BED", p, Face {
            mix: 0.55, tone: 0.3, drift: 0.4, wind: 0.65, sustain: 0.85, register: 0.0, air: 1.0, sway: 1.0,
        }),
        // The noise floor sings on its own: self-oscillating shimmer.
        face("AEOLIAN WIND", p, Face {
            mix: 0.7, tone: 0.55, drift: 0.6, wind: 0.95, sustain: 0.97, register: 3.0, air: 1.0, sway: 1.0,
        }),
        // Percussive sympathetic sparkle for picked parts.
        face("GLASS", p, Face {
            mix: 0.4, tone: 0.95, drift: 0.2, wind: 0.45, sustain: 0.5, register: 4.0, air: 1.0, sway: 0.0,
        }),
    ]
}
