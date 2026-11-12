//! Stave Aeolian — a sympathetic string harp: twelve plucked-string
//! voices, one per pitch class, that ring when you play their note in
//! any octave. WIND is velocity, SUSTAIN the decay, REGISTER the
//! octave; AIR and SWAY move it. Behind a verdigris-and-
//! brass face whose harp window shows the twelve strings themselves,
//! vibrating as they sing. **Start here**: this file is the plugin's
//! root; everything below is one click away.
//!
//! ```text
//! lib.rs      ← you are here: the crate's map and its export
//! params.rs   every knob, with its forever-stable id
//! presets.rs  factory presets
//! editor.rs   the face: parts from the stave-face design system
//! harp.rs     the harp window's picture: twelve live strings
//! engine/     the ENGINE half:
//!   mod.rs        the spine: `impl Plugin for Aeolian`, bind → render → observe
//!   detect.rs     Detector — twelve pitch-class bands → excitation
//!   strings.rs    Strings — twelve doubled Karplus–Strong voices
//!   ensemble.rs   Ensemble — CHORUS
//!   telemetry.rs  AeolianTaps — each string's ring + the clock → the face
//! tests/      the shared checks + THE PROPERTIES (properties.rs)
//! ```

mod editor;
mod engine;
mod harp;
mod params;
mod presets;

pub use engine::{Aeolian, AeolianTaps};
pub use params::AeolianParams;

stave::export!(Aeolian, vst3_class_ids: {
    component: [0x1109FB57, 0x8C770B3F, 0xDE422D02, 0x4153C104],
    controller: [0x88E6EFD3, 0xF14BF1D7, 0x2EADEF58, 0x23F72554],
});
