//! Stave Tolex. **Start here**: this file is the plugin's
//! root; everything below is one click away.
//!
//! ```text
//! lib.rs      ← you are here: the modules and the export
//! params.rs   every knob, with its forever-stable id (the full
//!               planned ledger is in plugins/tolex/docs/design.md)
//! editor.rs   the face (a speaker cabinet), a pure function of params
//! presets.rs  factory presets — BREAKDOWN is the flagship
//! curve.rs    ONE truth for picture and sound: the full response
//!               assembled from the engine's display twins, plus
//!               the baked population envelope (statistics only)
//! response.rs the response screen — live curve over the
//!               population territory (law-pinned: the picture
//!               cannot lie, tests/curve_properties.rs)
//! engine/     the ENGINE half — the cabinet in components:
//!   mod.rs      the orchestration (bind → render)
//!   motor.rs    the lumped electro-mechanical driver core (leaf)
//!   cabinet.rs  the open back's wrap-around rear path
//!   voice.rs    the modal fingerprint + position system
//! tests/      the shared checks (checks.rs: validation, no-alloc, perf,
//!               the face's laws) and THE LAWS (properties.rs — the physics
//!               promises)
//! ```
//!
//! The motor (T/S + Bl(x)/Cms(x) + source impedance + box) feeds the
//! cone's modal fingerprint: fixed mode set with ring-law Qs,
//! mic-position weights + comb + macro walk, drive-softened breakup.

pub mod curve;
mod editor;
mod engine;
mod params;
mod presets;
mod response;

pub use engine::{Tolex, TolexTaps};
pub use params::TolexParams;

stave::export!(Tolex, vst3_class_ids: {
    component: [0xEB796B35, 0xEDF8C2C4, 0xF9C39DCA, 0xBD83B4C7],
    controller: [0x5E690385, 0xBCCBE4AB, 0x9D21E046, 0x541F0AC3],
});
