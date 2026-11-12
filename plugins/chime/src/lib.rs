//! Stave Chime — a polyphonic sine/saw synth. **Start here**: this
//! file is the plugin's root; everything below is one click away.
//!
//! ```text
//! lib.rs      ← you are here: the crate's map and its export
//! params.rs   every knob, with its forever-stable id
//! editor.rs   the face, a pure function of params
//! engine/     the ENGINE half (split when the matrix arrived —
//!               the growth rule):
//!   mod.rs    `impl Plugin for Chime`: ChimeVoice × 16 under a
//!               VoiceRack + the event bridge
//!   matrix.rs the modulation semantics: routes, declared ranges,
//!               per-voice evaluation
//! tests/      the shared checks (checks.rs), editor gestures, and
//!               THE LAWS (pitch, velocity, release, steal,
//!               mid-block starts — tests/properties.rs)
//! ```
//!
//! STATUS: **the Voice form's first shipping consumer**
//! — sixteen voices, quietest-steal, ADSR,
//! sine or polyBLEP saw. Deliberately small: the product is the
//! FORM being provable in a real host.

mod editor;
mod engine;
mod params;

pub use engine::{Chime, ChimeTaps};
pub use params::ChimeParams;

stave::export!(Chime, vst3_class_ids: {
    component: [0x3E92B7C1, 0x4A5D4E88, 0xB1F0A6D2, 0x22810001],
    controller: [0x3E92B7C1, 0x4A5D4E88, 0xB1F0A6D2, 0x22810002],
});
