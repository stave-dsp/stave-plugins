//! The taps — where the loop is (0..1 of a turn: the face draws the
//! ribbon and its splice there, and turns the rollers by it) and how
//! fast the motor runs (1 = nominal).

use stave::prelude::*;

#[derive(Taps)]
pub struct RibbonTaps {
    /// Where the loop is, 0..1 of a turn.
    pub tape_loop: Tap,
    /// The motor's speed (1 = nominal).
    pub tape_speed: Tap,
}

impl RibbonTaps {
    pub(super) fn publish(&self, phase: f64, speed: f32) {
        self.tape_loop.set(phase);
        self.tape_speed.set(f64::from(speed));
    }
}
