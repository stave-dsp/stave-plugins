//! The taps — what heirloom reports out to its face: the live core
//! clock (the sag made visible), the wet tail's level, and the record's
//! SPIN angle (radians, wrapped) that the pedal's label turns by.

use stave::prelude::*;

#[derive(Taps)]
pub struct HeirloomTaps {
    /// The core clock ratio (1 = 32 kHz nominal; HOLD halves it).
    pub clock: Tap,
    /// The wet peak over a 60 dB window, 0..1.
    pub tail: Tap,
    /// The platter angle, radians in [0, 2π).
    pub spin: Tap,
}

impl HeirloomTaps {
    pub(super) fn publish(&self, clock: f32, wet_peak: f32, spin: f64) {
        self.spin.set(spin);
        self.clock.set(f64::from(clock));
        // 60 dB window → 0..1 (ears are logarithmic).
        let db = amp_to_db(wet_peak.max(1e-6));
        self.tail.set(f64::from((1.0 + db / 60.0).clamp(0.0, 1.0)));
    }
}
