//! Factory presets. RED / GREEN / BLUE are the pedal's own three
//! factory slots, read off the manual's knob diagrams (clock face
//! 7:00 = min … 5:00 = max, so each hour is a tenth of travel; TIME,
//! MOD RATE, SHIFT and RAMP are log knobs, converted through their
//! tapers). The rest are postcards for the programs the factory
//! slots skip. Every preset states the FULL surface.

use stave::prelude::*;

use crate::params::SagaParams;

/// One full surface, in pedal order. `mode` / `shape` are step indices.
struct Surface {
    mode: f64,
    feedback: f64,
    regen: f64,
    mod_depth: f64,
    mix: f64,
    x: f64,
    time: f64,
    tone: f64,
    width: f64,
    mod_rate: f64,
    shape: f64,
    shift: f64,
    ramp: f64,
}

fn surface(name: &'static str, p: &SagaParams, f: Surface) -> Preset {
    Preset::from_pairs(
        name,
        &[
            (p.mode.id(), f.mode), (p.feedback.id(), f.feedback), (p.regen.id(), f.regen),
            (p.mod_depth.id(), f.mod_depth), (p.mix.id(), f.mix), (p.x.id(), f.x), (p.time.id(), f.time),
            (p.tone.id(), f.tone), (p.width.id(), f.width), (p.mod_rate.id(), f.mod_rate),
            (p.mod_shape.id(), f.shape), (p.shift.id(), f.shift), (p.ramp.id(), f.ramp),
            (p.dive.id(), 0.0), (p.sync.id(), 0.0), (p.output.id(), 0.0),
        ],
    )
}

pub fn presets(p: &SagaParams) -> Vec<Preset> {
    vec![
        surface("INIT", p, Surface {
            mode: 0.0, feedback: 0.45, regen: 0.4, mod_depth: 0.25, mix: 0.5, x: 0.5,
            time: 450.0, tone: 0.75, width: 0.6, mod_rate: 0.6, shape: 0.0, shift: 0.5,
            ramp: 600.0,
        }),
        // Program I, reversed all the way into a reverse reverb that
        // nearly holds; slow random drift. (Manual: RED.)
        surface("FACTORY / RED", p, Surface {
            mode: 0.0, feedback: 0.65, regen: 0.95, mod_depth: 0.45, mix: 0.6, x: 0.95,
            time: 636.0, tone: 0.8, width: 0.7, mod_rate: 0.22, shape: 2.0, shift: 0.87,
            ramp: 500.0,
        }),
        // Program II, shimmer up, short reverb, square-stepped wobble,
        // DIVE drops a full octave instantly. (Manual: GREEN.)
        surface("FACTORY / GREEN", p, Surface {
            mode: 1.0, feedback: 0.8, regen: 0.8, mod_depth: 0.1, mix: 0.55, x: 0.2,
            time: 245.0, tone: 0.65, width: 1.0, mod_rate: 0.22, shape: 1.0, shift: 0.5,
            ramp: 50.0,
        }),
        // Program IV, both octaves feeding the reverse space, a long
        // forward tail; mostly wet. (Manual: BLUE.)
        surface("FACTORY / BLUE", p, Surface {
            mode: 3.0, feedback: 0.5, regen: 0.5, mod_depth: 0.05, mix: 0.8, x: 0.8,
            time: 394.0, tone: 0.45, width: 1.0, mod_rate: 0.71, shape: 0.0, shift: 0.87,
            ramp: 500.0,
        }),
        // Program III under fuzz territory: the octave-down murk.
        surface("III / UNDERTOW", p, Surface {
            mode: 2.0, feedback: 0.5, regen: 0.65, mod_depth: 0.3, mix: 0.5, x: 0.7,
            time: 520.0, tone: 0.5, width: 0.7, mod_rate: 0.3, shape: 0.0, shift: 0.5,
            ramp: 1500.0,
        }),
        // Program V slow: a sequencer climbing fourths and fifths.
        surface("V / STAIRWELL", p, Surface {
            mode: 4.0, feedback: 0.55, regen: 0.45, mod_depth: 0.1, mix: 0.45, x: 0.3,
            time: 375.0, tone: 0.7, width: 0.8, mod_rate: 0.5, shape: 0.0, shift: 0.5,
            ramp: 600.0,
        }),
        // Program V fast: the duel collapses into a harmonizer.
        surface("V / CHOIR", p, Surface {
            mode: 4.0, feedback: 0.3, regen: 0.3, mod_depth: 0.2, mix: 0.4, x: 0.85,
            time: 70.0, tone: 0.65, width: 0.9, mod_rate: 0.8, shape: 0.0, shift: 0.5,
            ramp: 600.0,
        }),
        // Program II as a pad: long airy tail, shimmer, a slow rise
        // armed on DIVE.
        surface("II / VEIL", p, Surface {
            mode: 1.0, feedback: 0.35, regen: 0.6, mod_depth: 0.2, mix: 0.6, x: 0.85,
            time: 600.0, tone: 0.7, width: 0.9, mod_rate: 0.3, shape: 0.0, shift: 2.0,
            ramp: 3000.0,
        }),
    ]
}
