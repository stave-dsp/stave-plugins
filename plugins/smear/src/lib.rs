//! Stave Smear — "Sympathetic Garden". **Start here**: this file is
//! the plugin's root; everything below is one click away.
//!
//! ```text
//! lib.rs      ← you are here: the crate's map and its export
//! params.rs   every knob, with its forever-stable id
//! editor.rs   the face (and the garden's picture), a pure function of params
//! engine/     the ENGINE half:
//!   mod.rs    the spine: `impl Plugin for Smear` (identity, layouts
//!               — stereo, with a 5.1 surround variant), the process
//!               loop + drift updates
//!   garden.rs the resonator lattice (one garden per lane) — a
//!               LEAF: dense cross-coupled cores stay whole
//! tests/      this project's properties: the garden suites, the
//!               surround lift bit-law, and the shared checks
//!               (checks.rs)
//! ```
//!
//! A field of tuned resonators (sympathetic strings) on a chromatic
//! pitch lattice. Your input excites them the way an open piano
//! rings when you sing into it; energy blooms across neighbouring
//! pitches (the "smear"), tunings drift so the garden is alive, and
//! a saturated regeneration path sustains into pads or drones.
//! Entirely TIME-DOMAIN: real filters, real outputs — it rings at
//! actual notes and cannot collapse into noise like a spectral
//! engine.
//!
//! Controls: SEED (input drive), SMEAR (neighbour coupling), BLOOM
//! (regeneration), DECAY (ring time), FLUX (drift rate), SHIMMER
//! (octave bloom), DRIVE (feedback warmth), MOTION (detune depth),
//! DEPTH (lane spread), TONE (HF damping), MIX, OUTPUT.

mod editor;
mod engine;
mod params;

pub use engine::{Smear, SmearTaps};
pub use params::SmearParams;

stave::export!(Smear, vst3_class_ids: {
    component: [0x5A7FDD1D, 0x6E5E4F88, 0x9D6D7E11, 0x55E76BF1],
    controller: [0x5A7FDD1D, 0x6E5E4F88, 0x9D6D7E11, 0x55E76BF2],
});
