//! Stave Opal — a 4-operator FM synth. **Start here**: this file is
//! the plugin's root; everything below is one click away.
//!
//! ```text
//! lib.rs      ← you are here: the map, the modules, the export
//! params.rs   every knob, with its forever-stable id (per-op ids
//!               blocked in tens, so the table reads like the panel)
//! diagram.rs  the algorithm picture (opal's OWN, a Display draw
//!               fn — bespoke visuals live in the project)
//! editor.rs   the face: algorithm plate + paged operators, LFO, MOD
//! presets.rs  factory presets, pure function of params
//! engine/     the plugin:
//!   mod.rs    `Opal` (identity, layout, `impl Plugin`): OpalVoice × 16
//!             under a VoiceRack + the event bridge
//!   operators.rs  the FM core LEAF: wiring-as-data algorithms,
//!               phase-mod math, the op-4 feedback loop
//! tests/      the shared checks + THE FM PROPERTIES (sidebands land where
//!               Chowning says; algorithms are audibly distinct;
//!               feedback grows teeth; velocity tracks brightness)
//! ```
//!
//! STATUS: **tier 2 of the instrument arc** — the
//! Operator-class consumer: per-voice control instancing at full
//! scale (4 envelopes × 16 voices), algorithms as data. Remaining
//! tier-2 legs (recorded, not lost): pitch-bend/mod-wheel plumbing,
//! keytrack cells, the interactive envelope-editor widget.

mod diagram;
mod editor;
mod engine;
mod params;
mod presets;

pub use engine::{Opal, OpalTaps};
pub use params::OpalParams;

stave::export!(Opal, vst3_class_ids: {
    component: [0x6B47D2A9, 0x91E34C77, 0x8AF5B0E1, 0x33920001],
    controller: [0x6B47D2A9, 0x91E34C77, 0x8AF5B0E1, 0x33920002],
});
