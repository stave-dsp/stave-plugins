//! The taps — what saga reports out to its face: the live core clock
//! (the dive made visible) and the wet tail's level for the history
//! strip.

use stave::prelude::*;

#[derive(Taps)]
pub struct SagaTaps {
    /// The core clock ratio (1 = 32 kHz nominal; the dive halves it).
    pub clock: Tap,
    /// The wet peak over a 60 dB window, 0..1.
    pub tail: Tap,
}

impl SagaTaps {
    pub(super) fn publish(&self, clock: f32, wet_peak: f32) {
        self.clock.set(f64::from(clock));
        // 60 dB window → 0..1 (ears are logarithmic).
        let db = amp_to_db(wet_peak.max(1e-6));
        self.tail.set(f64::from((1.0 + db / 60.0).clamp(0.0, 1.0)));
    }
}
