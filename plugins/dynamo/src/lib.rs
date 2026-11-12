//! Stave Dynamo — THE IMPOSSIBLE MACHINE: a
//! physical-modeling/evolving synth built from machinery that
//! never existed — rotor, wires with memory, one shared bridge,
//! one shared boiler, an escapement. **Start here**: this file is
//! the plugin's root; everything below is one click away.
//!
//! ```text
//! lib.rs      ← you are here: the crate's map and its export
//! params.rs   every knob, with its forever-stable id
//! editor.rs   the face, a pure function of params
//! machine.rs  the living machine diagram (a Display draw fn)
//! engine/     the ENGINE half (split when the matrix arrived —
//!               the growth rule):
//!   mod.rs    `Dynamo` (the Plugin): DynamoVoice × 16 under a
//!               VoiceRack + the event bridge
//!   matrix.rs the modulation semantics: routes, declared ranges,
//!               per-voice evaluation
//! tests/      checks.rs (every generic check, one macro), editor.rs,
//!               and THE LAWS (pitch, velocity, release,
//!               steal, mid-block starts — tests/properties.rs)
//! ```
//!
//! Sixteen voices under a quietest-steal rack, ADSR, the mod
//! matrix and the patch bay; each voice is a rotor driving a wire.

mod editor;
mod engine;
mod machine;
mod params;

pub use engine::{Dynamo, DynamoTaps};
pub use params::DynamoParams;

stave::export!(Dynamo, vst3_class_ids: {
    component: [0x8A29C64A, 0x4FC494A7, 0x0755565D, 0x7AFF1F4D],
    controller: [0x7263AFDA, 0xD9CADF89, 0x25C02CF8, 0x7B51E498],
});
