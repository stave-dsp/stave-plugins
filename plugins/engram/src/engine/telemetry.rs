//! The display's feed, PACKED: telemetry has 64 tap slots and the face
//! needs ~150 values, so several quantised values ride in each tap
//! (`crate::pack`, exact in an f64's 53-bit integer range). Out go: each
//! line's energy (48 dB window), every node's live position, what the
//! ear hears (chroma), every memory's attention weight, whether it holds
//! a memory (and a percussive one), how loudly it is recalling, how
//! NOVEL what you play is to it, the onset pulse, how tonal what you
//! play is, and the SPACE map's feed. Writes to detached taps (no
//! telemetry) are dropped.

use stave::prelude::*;

use super::bank::{Bank, SLOTS};
use super::geometry::Geometry;
use super::listen::CLASSES;
use super::room::{Room, LINES};
use super::space_model::SPACES;
use crate::{pack, TAP_HEARD_AT, TAP_SPACE, TAP_SPACE_INFO, TAP_ATTN, TAP_ENERGY, TAP_FILLED, TAP_HEARING, TAP_NOVELTY, TAP_PERC, TAP_POS, TAP_PULSE, TAP_RECALLING, TAP_TONAL};

/// The display's taps. A multi-tap group (`energy_0..3`, …) is one
/// packed value split across taps; the `crate::TAP_*` consts say where
/// each group starts in [`EngramTaps::ALL`] (field order).
#[derive(Taps)]
pub struct EngramTaps {
    pub energy_0: Tap,
    pub energy_1: Tap,
    pub energy_2: Tap,
    pub energy_3: Tap,
    pub pos_0: Tap,
    pub pos_1: Tap,
    pub pos_2: Tap,
    pub pos_3: Tap,
    pub pos_4: Tap,
    pub pos_5: Tap,
    pub pos_6: Tap,
    pub pos_7: Tap,
    pub hearing_0: Tap,
    pub hearing_1: Tap,
    pub hearing_2: Tap,
    pub attn_0: Tap,
    pub attn_1: Tap,
    pub attn_2: Tap,
    pub attn_3: Tap,
    pub attn_4: Tap,
    pub attn_5: Tap,
    pub attn_6: Tap,
    pub attn_7: Tap,
    pub attn_8: Tap,
    pub attn_9: Tap,
    pub attn_10: Tap,
    pub filled_0: Tap,
    pub filled_1: Tap,
    pub recalling: Tap,
    pub novelty: Tap,
    pub percussive_0: Tap,
    pub percussive_1: Tap,
    pub pulse: Tap,
    pub tonal: Tap,
    pub space: Tap,
    pub space_info: Tap,
    pub heard_at_0: Tap,
    pub heard_at_1: Tap,
    pub heard_at_2: Tap,
    pub heard_at_3: Tap,
    pub heard_at_4: Tap,
    pub heard_at_5: Tap,
    pub heard_at_6: Tap,
    pub heard_at_7: Tap,
    pub heard_at_8: Tap,
    pub heard_at_9: Tap,
    pub heard_at_10: Tap,
    pub heard_at_11: Tap,
    pub heard_at_12: Tap,
    pub heard_at_13: Tap,
    pub heard_at_14: Tap,
    pub heard_at_15: Tap,
    pub heard_at_16: Tap,
    pub heard_at_17: Tap,
    pub heard_at_18: Tap,
    pub heard_at_19: Tap,
    pub heard_at_20: Tap,
    pub heard_at_21: Tap,
}

/// How many taps the display reads.
pub const TAPS: usize = 58;

fn db01(x: f32) -> f64 {
    let db = 20.0 * x.max(1e-7).log10();
    f64::from((1.0 + db / 48.0).clamp(0.0, 1.0))
}

impl EngramTaps {
    /// Every tap, in field order (as [`EngramTaps::ALL`]).
    fn slots(&self) -> [&Tap; TAPS] {
        [&self.energy_0, &self.energy_1, &self.energy_2, &self.energy_3, &self.pos_0, &self.pos_1, &self.pos_2, &self.pos_3, &self.pos_4, &self.pos_5, &self.pos_6, &self.pos_7, &self.hearing_0, &self.hearing_1, &self.hearing_2, &self.attn_0, &self.attn_1, &self.attn_2, &self.attn_3, &self.attn_4, &self.attn_5, &self.attn_6, &self.attn_7, &self.attn_8, &self.attn_9, &self.attn_10, &self.filled_0, &self.filled_1, &self.recalling, &self.novelty, &self.percussive_0, &self.percussive_1, &self.pulse, &self.tonal, &self.space, &self.space_info, &self.heard_at_0, &self.heard_at_1, &self.heard_at_2, &self.heard_at_3, &self.heard_at_4, &self.heard_at_5, &self.heard_at_6, &self.heard_at_7, &self.heard_at_8, &self.heard_at_9, &self.heard_at_10, &self.heard_at_11, &self.heard_at_12, &self.heard_at_13, &self.heard_at_14, &self.heard_at_15, &self.heard_at_16, &self.heard_at_17, &self.heard_at_18, &self.heard_at_19, &self.heard_at_20, &self.heard_at_21]
    }

    pub(super) fn publish(&self, room: &Room, geometry: &Geometry, bank: &Bank, recalling: f32, pulse: f32, tonal: f32) {
        let taps = self.slots();
        let set = |k: usize, v: f64| taps[k].set(v);
        for g in 0..LINES / 4 {
            let v: [f64; 4] = core::array::from_fn(|j| db01(room.energy(4 * g + j)));
            set(TAP_ENERGY + g, pack(&v, 13));
        }
        for g in 0..2 * LINES / 4 {
            let v: [f64; 4] = core::array::from_fn(|j| {
                let k = 4 * g + j;
                f64::from(geometry.pos(k / 2)[k % 2])
            });
            set(TAP_POS + g, pack(&v, 13));
        }
        let q = &bank.query().chroma;
        for g in 0..CLASSES / 4 {
            let v: [f64; 4] = core::array::from_fn(|j| f64::from(q[4 * g + j]));
            set(TAP_HEARING + g, pack(&v, 13));
        }
        for g in 0..SLOTS.div_ceil(6) {
            let v: [f64; 6] = core::array::from_fn(|j| {
                let k = 6 * g + j;
                if k < SLOTS { f64::from(bank.weight(k)).sqrt() } else { 0.0 }
            });
            set(TAP_ATTN + g, pack(&v, 8));
        }
        for half in 0..2 {
            let (mut m, mut p) = (0u64, 0u64);
            for b in 0..32 {
                if bank.filled(32 * half + b) {
                    m |= 1 << b;
                }
                if bank.percussive(32 * half + b) {
                    p |= 1 << b;
                }
            }
            set(TAP_FILLED + half, m as f64);
            set(TAP_PERC + half, p as f64);
        }
        set(TAP_PULSE, f64::from(pulse.clamp(0.0, 1.0)));
        set(TAP_TONAL, f64::from(tonal.clamp(0.0, 1.0)));
        set(TAP_RECALLING, db01(recalling));
        set(TAP_NOVELTY, f64::from(bank.novelty()));
    }

    /// The SPACE window's feed: the room on the map, where memory pulls
    /// it, the nearest real space, the decay; every memory's room.
    pub(super) fn publish_space(&self, bank: &Bank, live: [f32; 2], near: usize, pull_at: [f32; 2], pull: f32, t60: f32) {
        let taps = self.slots();
        let set = |k: usize, v: f64| taps[k].set(v);
        let f = |v: f32| f64::from(v);
        set(TAP_SPACE, pack(&[f(live[0]), f(live[1]), f(pull_at[0]), f(pull_at[1])], 13));
        // T60 on a log scale, 0.1 … 30 s.
        let t = f64::from((t60.max(0.1) / 0.1).ln() / (300f32).ln());
        set(TAP_SPACE_INFO, pack(&[near as f64 / (SPACES - 1).max(1) as f64, f(pull), t], 13));
        for g in 0..(2 * SLOTS).div_ceil(6) {
            let v: [f64; 6] = core::array::from_fn(|j| {
                let k = 6 * g + j;
                if k < 2 * SLOTS { f(bank.heard_at(k / 2)[k % 2]) } else { 0.0 }
            });
            set(TAP_HEARD_AT + g, pack(&v, 8));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `slots` and `ALL` agree tap for tap (both are field order).
    #[test]
    fn the_slots_are_the_ids_in_order() {
        assert_eq!(EngramTaps::ALL.len(), TAPS);
        let telemetry = Telemetry::new();
        let taps = EngramTaps::new(&telemetry);
        for (k, tap) in taps.slots().iter().enumerate() {
            tap.set(k as f64);
        }
        for (k, id) in EngramTaps::ALL.iter().enumerate() {
            assert_eq!(telemetry.value(*id), Some(k as f64), "{}", id.name());
        }
    }
}
