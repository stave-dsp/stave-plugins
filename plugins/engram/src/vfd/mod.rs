//! The vacuum-fluorescent displays that show engram THINKING, each a
//! glass window on the face:
//!
//! * [`memory`] — the bank as an 8 × 8 grid (a cell glows faintly when
//!   it holds a memory, cyan a tonal one, amber a percussive one, and
//!   blazes with the attention it is paid right now); the query's twelve
//!   pitch classes with the PULSE lamp; the RECALL and NOVELTY strips.
//! * [`neurons`] — the room's sixteen delay lines, glowing with their
//!   energy, tethered to where plasticity has pulled them; grab one and
//!   drag it (radius = a line's length, closeness = coupling).
//! * [`space`] — the MAP of the ≈700 real reverbs the room learned:
//!   click to put the cursor; the live room glides after it, pulled
//!   (amber) toward where the recalled memories were heard.
//!
//! All three read the packed display taps (`crate::FACE_TAPS`) through
//! [`Feed`]. The SPACE map draws y-up (a face pad's convention); the
//! NEURONS field draws y-down, as the pedal face did (its pad area is
//! flipped to match). Within a window every coordinate maps the same
//! way, so what looks close is close.

pub mod memory;
pub mod neurons;
pub mod space;

use stave_core::face::prelude::*;

use crate::{
    unpack, FACE_TAPS, TAP_ATTN, TAP_ENERGY, TAP_FILLED, TAP_HEARD_AT, TAP_HEARING, TAP_NOVELTY, TAP_PERC, TAP_POS,
    TAP_PULSE, TAP_RECALLING, TAP_SPACE, TAP_SPACE_INFO, TAP_TONAL,
};

/// Every window is this tall (the screen, inside its bezel).
pub const SCREEN_H: f64 = 224.0;
/// Memory slots in the bank, and the neurons in the room.
pub const SLOTS: usize = 64;
pub const LINES: usize = 16;

/// Percussive memories glow amber (VFDs came in both).
const AMBER: Color = Color::rgba(1.0, 0.72, 0.3, 1.0);

/// The theme's phosphor at alpha `a`.
pub fn glow(th: &Theme, a: f64) -> Color {
    let p = th.phosphor;
    Color::rgba(p.r, p.g, p.b, a.clamp(0.0, 1.0))
}

pub fn amber(a: f64) -> Color {
    Color::rgba(AMBER.r, AMBER.g, AMBER.b, a.clamp(0.0, 1.0))
}

/// A segment that is merely there: unlit phosphor, as on a real VFD.
pub fn unlit(th: &Theme) -> Color {
    glow(th, 0.055)
}

/// A label printed in the glass (small tracked caps, in phosphor).
pub fn label(f: &Frame<'_>, s: &mut Scene, at: Pt, text: impl Into<String>, a: f64, align: TextAlign) {
    let ty = &f.theme.type_;
    s.text_in(at, text, 7.0, glow(f.theme, a), ty.caption.align(align));
}

pub use stave_core::ui::scene::{Pt, TextAlign};

/// The display's decoded feed.
pub struct Feed {
    pub energy: Vec<f64>,
    pub pos: Vec<f64>,
    pub hearing: Vec<f64>,
    pub attn: Vec<f64>,
    pub filled: u64,
    pub percussive: u64,
    pub recalling: f64,
    pub novelty: f64,
    pub pulse: f64,
    pub tonal: f64,
    /// The room on the map, where memory pulls it, how hard; the
    /// nearest real space (index); the T60 (s); every memory's room.
    pub live: (f64, f64),
    pub pull_at: (f64, f64),
    pub pull: f64,
    pub near: f64,
    pub t60: f64,
    pub heard_at: Vec<f64>,
}

impl Feed {
    pub fn read(f: &Frame<'_>) -> Self {
        Self::from_probes(|k| f.probe(FACE_TAPS[k]))
    }

    pub fn from_probes(probe: impl Fn(usize) -> f64) -> Self {
        let gather = |from: usize, taps: usize, per: usize, bits: u32| -> Vec<f64> {
            (0..taps).flat_map(|t| unpack(probe(from + t), per, bits)).collect()
        };
        let bits = |v: f64| if v.is_finite() && v >= 0.0 { v as u64 & 0xFFFF_FFFF } else { 0 };
        let unit = |k: usize| {
            let v = probe(k);
            if v.is_finite() {
                v.clamp(0.0, 1.0)
            } else {
                0.0
            }
        };
        let space = unpack(probe(TAP_SPACE), 4, 13);
        let info = unpack(probe(TAP_SPACE_INFO), 3, 13);
        Feed {
            energy: gather(TAP_ENERGY, 4, 4, 13),
            pos: gather(TAP_POS, 8, 4, 13),
            hearing: gather(TAP_HEARING, 3, 4, 13),
            attn: gather(TAP_ATTN, 11, 6, 8),
            filled: bits(probe(TAP_FILLED)) | (bits(probe(TAP_FILLED + 1)) << 32),
            percussive: bits(probe(TAP_PERC)) | (bits(probe(TAP_PERC + 1)) << 32),
            pulse: unit(TAP_PULSE),
            tonal: unit(TAP_TONAL),
            recalling: unit(TAP_RECALLING),
            novelty: unit(TAP_NOVELTY),
            live: (space[0], space[1]),
            pull_at: (space[2], space[3]),
            pull: info[1],
            near: info[0],
            t60: 0.1 * 300f64.powf(info[2]),
            heard_at: gather(TAP_HEARD_AT, 22, 6, 8),
        }
    }

    pub fn held(&self, k: usize) -> bool {
        self.filled >> k & 1 == 1
    }

    pub fn percussive(&self, k: usize) -> bool {
        self.percussive >> k & 1 == 1
    }

    pub fn attention(&self, k: usize) -> f64 {
        self.attn.get(k).copied().unwrap_or(0.0)
    }

    /// The ink a memory glows in: amber when percussive, else phosphor.
    pub fn ink(&self, th: &Theme, k: usize, a: f64) -> Color {
        if self.percussive(k) {
            amber(a)
        } else {
            glow(th, a)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_packed_feed_decodes() {
        let mut probes = vec![0.0; FACE_TAPS.len()];
        probes[TAP_FILLED] = 0b101 as f64;
        probes[TAP_PERC + 1] = 1.0;
        probes[TAP_ATTN] = crate::pack(&[0.0, 0.0, 1.0, 0.0, 0.0, 0.0], 8);
        probes[TAP_SPACE] = crate::pack(&[0.25, 0.5, 0.75, 1.0], 13);
        let f = Feed::from_probes(|k| probes[k]);
        assert!(f.held(0) && !f.held(1) && f.held(2));
        assert!(f.percussive(32) && !f.percussive(0));
        assert!((f.attention(2) - 1.0).abs() < 1e-9);
        assert!((f.live.0 - 0.25).abs() < 1e-3 && (f.pull_at.1 - 1.0).abs() < 1e-3);
        assert_eq!(f.energy.len(), LINES);
        assert_eq!(f.pos.len(), 2 * LINES);
        assert!(f.attn.len() >= SLOTS && f.heard_at.len() >= 2 * SLOTS);
    }
}
