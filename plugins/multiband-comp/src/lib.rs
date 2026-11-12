//! Stave Multiband — a 3-band compressor. **Start here**: this file
//! is the plugin's root; everything below is one click away.
//!
//! ```text
//! lib.rs        the crate root and the export
//! params.rs     every knob, with its forever-stable id
//! style.rs      compressor styles (VCA/FET/OPTO/VMU) and their
//!               program dependence
//! detector.rs   a band's detector: HPF, peak/RMS, retimed follower
//! saturation.rs the per-band saturator
//! editor.rs     the face, a pure function of params
//! views.rs      the face's screens: spectrum, GR, transfer curves
//! engine/
//!   mod.rs      the plugin: identity, layouts (stereo + an
//!               external-sidechain aux), the split, the bands, the sum
//!   band.rs     one band: controls, gain, saturation, blend
//!   splitter.rs the LR4 band split (main path and sidechain)
//!   dynamics.rs the gain computer
//! tests/        checks, sidechain, editor
//! ```
//!
//! Topology: two Linkwitz-Riley 4th-order crossovers split the input
//! into low/mid/high; each band runs an independent feedforward
//! compressor with per-band character; the bands sum back.

pub mod detector;
mod editor;
mod engine;
mod params;
#[doc(hidden)]
pub mod saturation;
pub mod style;
mod views;

pub use engine::{MultibandComp, MultibandTaps};
pub use params::{BandParams, MbParams};

stave::export!(MultibandComp, vst3_class_ids: {
    component: [0x5A7FCC1C, 0x6E5E4F88, 0x9D6D7E11, 0x55E76BE1],
    controller: [0x5A7FCC1C, 0x6E5E4F88, 0x9D6D7E11, 0x55E76BE2],
});
