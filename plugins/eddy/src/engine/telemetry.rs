//! What eddy reports out to its face: the wave view's published cycle,
//! its live phase cursor, and the tail's level.

use stave::prelude::*;
use stave::telemetry::CURVE_POINTS;

/// The face's live values, by name (`EddyTaps::LFO_PHASE`, `::TAIL`).
#[derive(Taps)]
pub struct EddyTaps {
    /// The wave view's cursor: phase 0..1 through the cycle.
    pub lfo_phase: Tap,
    /// The tail view's feed: the block's wet peak through a 60 dB
    /// window, 0..1.
    pub tail: Tap,
}

impl EddyTaps {
    /// The tail view's feed: the block's wet peak, mapped through a
    /// 60 dB window to 0..1 (ears are logarithmic — a linear peak
    /// would show only the first repeat or two).
    pub(super) fn publish_tail(&self, wet_peak: f32) {
        let db = 20.0 * wet_peak.max(1e-6).log10();
        self.tail.set(f64::from((1.0 + db / 60.0).clamp(0.0, 1.0)));
    }
}

/// Publish the shaped cycle for the wave view: `draw` fills the
/// telemetry's curve slots (block rate, only when the recipe moved).
pub(super) fn publish_cycle(telemetry: &Telemetry, draw: impl FnOnce(&mut [f32])) {
    let mut ys = [0.0f32; CURVE_POINTS];
    draw(&mut ys);
    telemetry.publish_curve(&ys);
}
