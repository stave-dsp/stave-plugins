//! Stave Heirloom — saga's reverse machines (reverse delays and
//! reverbs on a clock you can pull) played back off a worn record: a
//! Warped Vinyl-style platter wobble and a Generation Loss-style
//! wear stage, behind a six-knob pedal face. saga's docs are the
//! map. **Start here**: this file is the plugin's root; everything
//! below is one click away.
//!
//! ```text
//! lib.rs      ← you are here: the modules and the export
//! params.rs   every knob, with its forever-stable id
//! presets.rs  factory presets
//! editor.rs   the face: parts on a cream pedal + the spinning record
//! engine/     the ENGINE half:
//!   mod.rs        Heirloom: identity + the spine (bind → render → observe)
//!   motion.rs     Motion — the core clock: base, HOLD, MOD LFO
//!   clock.rs      ClockDomain — host rate ⇄ the core clock
//!   programs.rs   Programs — the reverse machines
//!   reverse.rs    ReverseLine — a line read backward
//!   tank.rs       Tank — the 8-line FDN reverb
//!   transpose.rs  Transposer — the two-head pitch shifter
//!   path.rs       AnalogPath — the feedback loops' analog legs
//!   wet.rs        Wet — tone, width, the soft ceiling
//!   media.rs      Media — WOBBLE + AGE, the wet path's record
//!   telemetry.rs  HeirloomTaps — clock, tail, spin out
//! tests/      the shared checks + THE PROPERTIES (properties.rs)
//! ```

mod clip;
mod editor;
mod engine;
mod params;
mod presets;

pub use engine::{Heirloom, HeirloomTaps};
pub use params::HeirloomParams;

stave::export!(Heirloom, vst3_class_ids: {
    component: [0x68E5A59A, 0x63E215C2, 0x02E460FC, 0x637F31D7],
    controller: [0x997883AB, 0xFACDA952, 0x33F55080, 0xD9A58D26],
});
