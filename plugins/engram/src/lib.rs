//! Stave Engram — a reverb with an associative memory. As you play it
//! remembers moments of your playing (¾-second grains, keyed by their
//! harmony, timbre and groove — chords and drums alike), and its tail
//! is RETRIEVED as much as reverberated: what you play now is a query,
//! attention (a softmax over similarity, as in a transformer) picks the
//! past moments most like it, and those memories play — seamless to
//! shattered (TEXTURE), answering your hits — into a room
//! whose sixteen delay-line neurons you can drag, and which rewires
//! itself (Hebbian plasticity). Play something you played before and
//! the room answers with that earlier moment. An engram is a memory's
//! physical trace.
//! **Start here**: this file is the plugin's root; everything below is
//! one click away.
//!
//! ```text
//! lib.rs      ← you are here: the modules, the display's packed taps,
//!               the export — no behavior
//! params.rs   every knob, with its forever-stable id
//! presets.rs  factory presets
//! editor.rs   the face: parts from the stave-face design system
//! vfd/        the glass windows' pictures: MEMORY, NEURONS, SPACE
//! engine/     the ENGINE half:
//!   mod.rs        Engram (the Plugin) — hear → remember → attend → recall → room → mix
//!   room.rs       Room — the sixteen-line reservoir, shaped by geometry
//!   geometry.rs   Geometry — the drawn + learned node positions
//!   listen.rs     Listen — the ear: your harmony as a fingerprint
//!   bank.rs       Bank — the associative memory (attention recall)
//!   telemetry.rs  EngramTaps — the display's (packed) feed
//! tests/      the shared checks + THE PROPERTIES (properties.rs)
//! ```

mod editor;
mod engine;
mod params;
mod presets;
mod vfd;

pub use engine::{Engram, EngramTaps};
pub use params::EngramParams;

/// The display's field is this many heights wide (geometry is measured
/// as the face draws it, so what looks close IS close).
pub const FIELD_ASPECT: f64 = 1.9;

/// Pack up to 53 bits' worth of quantised 0..1 values into one f64
/// tap value (`bits` each, exact — integers below 2^53 are exact in f64).
/// Telemetry has 64 tap slots; the face needs ~150 values.
pub fn pack(vals: &[f64], bits: u32) -> f64 {
    let q = ((1u64 << bits) - 1) as f64;
    let mut acc = 0u64;
    for (i, v) in vals.iter().enumerate() {
        acc |= ((v.clamp(0.0, 1.0) * q).round() as u64) << (i as u32 * bits);
    }
    acc as f64
}

/// The inverse of [`pack`]: `n` values of `bits` each.
pub fn unpack(v: f64, n: usize, bits: u32) -> Vec<f64> {
    let q = (1u64 << bits) - 1;
    let x = if v.is_finite() && v >= 0.0 { v as u64 } else { 0 };
    (0..n).map(|i| ((x >> (i as u32 * bits)) & q) as f64 / q as f64).collect()
}

// The TAP_* consts: where each group of the display's taps starts in
// `EngramTaps::ALL` (field order; `EngramTaps::slots` writes the same).

/// 16 line energies, 4 per tap (13-bit).
pub const TAP_ENERGY: usize = 0;
/// 32 node coordinates (x, y interleaved), 4 per tap (13-bit).
pub const TAP_POS: usize = 4;
/// 12 chroma values (what it hears), 4 per tap (13-bit).
pub const TAP_HEARING: usize = 12;
/// 64 attention weights (√w, 8-bit), 6 per tap.
pub const TAP_ATTN: usize = 15;
/// Which of the 64 slots hold a memory: two 32-bit masks.
pub const TAP_FILLED: usize = 26;
/// How loudly it is recalling (48 dB window), and how novel what you
/// play is to it (1 − best similarity).
pub const TAP_RECALLING: usize = 28;
pub const TAP_NOVELTY: usize = 29;
/// Which slots hold a PERCUSSIVE memory: two 32-bit masks.
pub const TAP_PERC: usize = 30;
/// The onset flash (1 on a hit, decaying), and how tonal you are.
pub const TAP_PULSE: usize = 32;
pub const TAP_TONAL: usize = 33;
/// The room on the SPACE map and where memory pulls it: (live x, live
/// y, pull x, pull y), 13-bit.
pub const TAP_SPACE: usize = 34;
/// (nearest real space / (SPACES − 1), pull strength, T60 on a log
/// 0.1 … 30 s scale), 13-bit.
pub const TAP_SPACE_INFO: usize = 35;
/// Where each of the 64 memories was heard on the map (x, y
/// interleaved), 8-bit, 6 per tap.
pub const TAP_HEARD_AT: usize = 36;

stave::export!(Engram, vst3_class_ids: {
    component: [0x9D173A5C, 0xD2BFAA66, 0xF030659F, 0xAA5CF54C],
    controller: [0x98623D49, 0xE19F9168, 0xA90E8C84, 0xF7C87EDC],
});
