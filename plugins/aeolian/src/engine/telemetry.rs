//! The taps — each string's ring level, on a 48 dB window (0 = still,
//! 1 = full), one tap per pitch class, plus the harp's CLOCK (seconds
//! of audio played): the face draws each string vibrating at its
//! level, in time.

use stave::prelude::*;

use super::detect::CLASSES;

const WINDOW_DB: f32 = 48.0;

/// Everything the face reads: the twelve strings, C … B, then the clock.
#[derive(Taps)]
pub struct AeolianTaps {
    pub ring_c: Tap,
    pub ring_cs: Tap,
    pub ring_d: Tap,
    pub ring_ds: Tap,
    pub ring_e: Tap,
    pub ring_f: Tap,
    pub ring_fs: Tap,
    pub ring_g: Tap,
    pub ring_gs: Tap,
    pub ring_a: Tap,
    pub ring_as: Tap,
    pub ring_b: Tap,
    /// Seconds of audio played (wrapped hourly).
    pub clock: Tap,
}

impl AeolianTaps {
    /// The twelve string taps, C … B — each string's ring level (0..1).
    pub const RINGS: [TapId; CLASSES] = [
        Self::RING_C,
        Self::RING_CS,
        Self::RING_D,
        Self::RING_DS,
        Self::RING_E,
        Self::RING_F,
        Self::RING_FS,
        Self::RING_G,
        Self::RING_GS,
        Self::RING_A,
        Self::RING_AS,
        Self::RING_B,
    ];

    fn rings(&self) -> [&Tap; CLASSES] {
        [
            &self.ring_c,
            &self.ring_cs,
            &self.ring_d,
            &self.ring_ds,
            &self.ring_e,
            &self.ring_f,
            &self.ring_fs,
            &self.ring_g,
            &self.ring_gs,
            &self.ring_a,
            &self.ring_as,
            &self.ring_b,
        ]
    }

    pub(super) fn publish(&self, levels: &[f32; CLASSES], clock: f64) {
        self.clock.set(clock);
        for (tap, l) in self.rings().into_iter().zip(levels) {
            let db = 20.0 * l.max(1e-7).log10();
            tap.set(f64::from((1.0 + db / WINDOW_DB).clamp(0.0, 1.0)));
        }
    }
}
