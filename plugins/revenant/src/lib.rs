//! Stave Revenant — a listening machine. Inside is an echo-state
//! network (a reservoir computer, written from scratch) whose readout
//! learns, online, to predict what you will play a few milliseconds
//! from now. What you hear is its guess: a ghost a step ahead of you.
//! Let it dream and it hears itself instead of you — and keeps
//! playing, in the shape of what it learned, when you stop. Behind a
//! lab-instrument face: a porthole onto the living network, and a
//! SURPRISE meter. **Start here**: this file is the plugin's root;
//! everything below is one click away.
//!
//! ```text
//! lib.rs      ← you are here: the crate's map and its export
//! params.rs   every knob, with its forever-stable id
//! presets.rs  factory presets
//! editor.rs   the face: parts on hammertone + the porthole and meter
//! engine/     the ENGINE half:
//!   mod.rs        the spine: `impl Plugin for Revenant`, decimate →
//!                 think → interpolate → mix
//!   brain.rs      Brain — the reservoir and its online readout
//!   telemetry.rs  RevenantTaps — surprise and neuron activity
//! tests/      the shared checks + THE PROPERTIES (properties.rs)
//! ```

mod editor;
mod engine;
mod params;
mod presets;

pub use engine::{Revenant, RevenantTaps};
pub use params::RevenantParams;

stave::export!(Revenant, vst3_class_ids: {
    component: [0x6774941E, 0x457E446D, 0x62BC0705, 0x061C8A4A],
    controller: [0xCE935155, 0x1A4A48A1, 0x6524FDBE, 0xA4DFB0B5],
});
