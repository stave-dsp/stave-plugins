//! Stave Ribbon — an impossible tape echo, built the way the machine
//! is built: a loop of tape modelled in TAPE coordinates (erase head,
//! record head, three playback heads, a motor with inertia), a 12AX7
//! in front of the record head, a dispersive two-spring tank. The
//! impossible part is the ERASE knob: turn it down and the loop keeps
//! what it held — every pass records on top of the last, each
//! generation duller, sound-on-sound into a slowly decaying archive.
//! Behind an oxblood face whose smoked window shows the loop running.
//! **Start here**: this file is the plugin's root; everything below is
//! one click away.
//!
//! ```text
//! lib.rs      ← you are here: the crate's map and its export
//! params.rs   every knob, with its forever-stable id
//! presets.rs  factory presets
//! editor.rs   the face: parts on oxblood + the running tape window
//! engine/     the ENGINE half:
//!   mod.rs        the spine: `impl Plugin for Ribbon`, bind → render → observe
//!   tape.rs       Tape — the loop, the heads, the motor
//!   preamp.rs     Preamp — the 12AX7 into the record head
//!   spring.rs     Spring — the dispersive two-spring tank
//!   telemetry.rs  RibbonTaps — where the loop is, how fast it runs
//! tests/      the shared checks + THE PROPERTIES (properties.rs)
//! ```

mod editor;
mod engine;
mod params;
mod presets;

pub use engine::{Ribbon, RibbonTaps};
pub use params::RibbonParams;

stave::export!(Ribbon, vst3_class_ids: {
    component: [0x57D986BC, 0x6E8207EB, 0x51A73C99, 0xD1CD583C],
    controller: [0x78955EAB, 0x3D6B1CFC, 0xE191FB7F, 0xCF075A71],
});
