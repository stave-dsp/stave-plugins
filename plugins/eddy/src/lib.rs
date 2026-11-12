//! Stave Eddy — a delay/chorus. **Start here**: this file is the
//! plugin's root; everything below is one click away.
//!
//! ```text
//! lib.rs      ← you are here: the map and the export
//! params.rs   every knob, with its forever-stable id
//! editor.rs   the face, a pure function of params
//! presets.rs  factory presets, pure function of params
//! engine/     the ENGINE half — components, one per concept:
//!   mod.rs    the spine: `Eddy` + its `impl Plugin` (identity,
//!             process in three named phases, face, presets)
//!   echo.rs   Echo — the M/S delay lanes + the feedback law
//!   motion.rs Motion — the LFO × recipe (pristine/Juno/Vibe over
//!               the kernel basis: shape ∘ lag ∘ drift)
//!   telemetry.rs EddyTaps — the wave cycle + phase cursor out
//! tests/      checks.rs (every generic check, one line), editor.rs
//!             (the tabs), and the project's OWN laws (delay,
//!             algorithms, diffusion, tail)
//! ```
//!
//! STATUS: a delay/chorus. Time (tape-bend glide), feedback
//! (capped below runaway), mix (bit-dry at zero, by law), and
//! MOTION: rate/depth plus a character menu whose vintage names are
//! RECIPES over kernel primitives, with the live wave on screen.

mod editor;
mod engine;
mod params;
mod presets;

pub use engine::{Eddy, EddyTaps};
pub use params::EddyParams;

stave::export!(Eddy, vst3_class_ids: {
    component: [0x5A7FEDD9, 0x6E5E4F88, 0x9D6D7E11, 0x55E90001],
    controller: [0x5A7FEDD9, 0x6E5E4F88, 0x9D6D7E11, 0x55E90002],
});
