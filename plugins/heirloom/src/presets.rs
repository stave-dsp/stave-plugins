//! Factory presets — corners of the character space, each stating
//! the FULL face (knobs, toggles, footswitch) so loading one never
//! inherits a stray setting.

use stave::prelude::*;

use crate::params::HeirloomParams;

/// One full surface. `machine` / `warp` / `media` are step indices.
struct Surface {
    time: f64,
    repeats: f64,
    mix: f64,
    wobble: f64,
    rpm: f64,
    age: f64,
    machine: f64,
    warp: f64,
    media: f64,
}

fn surface(name: &'static str, p: &HeirloomParams, f: Surface) -> Preset {
    Preset::from_pairs(
        name,
        &[
            (p.time.id(), f.time), (p.repeats.id(), f.repeats), (p.mix.id(), f.mix),
            (p.wobble.id(), f.wobble), (p.rpm.id(), f.rpm), (p.age.id(), f.age),
            (p.machine.id(), f.machine), (p.warp.id(), f.warp), (p.media.id(), f.media),
            (p.hold.id(), 0.0), (p.sync.id(), 0.0), (p.output.id(), 0.0),
        ],
    )
}

pub fn presets(p: &HeirloomParams) -> Vec<Preset> {
    vec![
        // The defaults: a clean record (AGE is the wear you add).
        surface("INIT", p, Surface {
            time: 450.0, repeats: 0.5, mix: 0.45, wobble: 0.3, rpm: 0.8, age: 0.0,
            machine: 0.0, warp: 0.0, media: 0.0,
        }),
        // A reverse swell off a dusty LP: slow warp, crackle under it.
        surface("ATTIC LP", p, Surface {
            time: 640.0, repeats: 0.6, mix: 0.55, wobble: 0.45, rpm: 0.55, age: 0.45,
            machine: 0.0, warp: 1.0, media: 0.0,
        }),
        // Octave shimmer dubbed to a cassette that's been in a hot car.
        surface("HOT CAR", p, Surface {
            time: 300.0, repeats: 0.7, mix: 0.5, wobble: 0.35, rpm: 1.6, age: 0.6,
            machine: 1.0, warp: 2.0, media: 1.0,
        }),
        // BLOOM on a third-generation VHS dub: wide, narrow, wandering.
        surface("HOME MOVIE", p, Surface {
            time: 800.0, repeats: 0.55, mix: 0.7, wobble: 0.25, rpm: 0.35, age: 0.7,
            machine: 2.0, warp: 2.0, media: 2.0,
        }),
        // Mostly Warped Vinyl: half wet, so the platter beats against
        // the dry; one far-off reverse repeat underneath.
        surface("SEASICK", p, Surface {
            time: 1200.0, repeats: 0.0, mix: 0.5, wobble: 0.7, rpm: 2.4, age: 0.1,
            machine: 0.0, warp: 0.0, media: 0.0,
        }),
        // Mostly Generation Loss: all wet, through a dying deck; one
        // far-off reverse repeat.
        surface("DYING DECK", p, Surface {
            time: 1200.0, repeats: 0.0, mix: 1.0, wobble: 0.1, rpm: 0.5, age: 0.85,
            machine: 0.0, warp: 2.0, media: 1.0,
        }),
        // Reverse loops held near forever, barely worn — a drone bed.
        surface("ETERNAL", p, Surface {
            time: 1000.0, repeats: 0.93, mix: 0.75, wobble: 0.2, rpm: 0.2, age: 0.2,
            machine: 2.0, warp: 0.0, media: 0.0,
        }),
    ]
}
