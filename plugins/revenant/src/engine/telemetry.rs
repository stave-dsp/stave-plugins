//! The taps — SURPRISE (how wrong its last guesses were, relative to
//! what you played: 0 calm … 1 lost) and the live activity of the
//! neurons the face shows (the first 24 of the network).

use stave::prelude::*;

use super::brain::Brain;

#[derive(Taps)]
pub struct RevenantTaps {
    /// How wrong its recent guesses were, 0 (calm) … 1 (lost).
    pub surprise: Tap,
    pub neuron_0: Tap,
    pub neuron_1: Tap,
    pub neuron_2: Tap,
    pub neuron_3: Tap,
    pub neuron_4: Tap,
    pub neuron_5: Tap,
    pub neuron_6: Tap,
    pub neuron_7: Tap,
    pub neuron_8: Tap,
    pub neuron_9: Tap,
    pub neuron_10: Tap,
    pub neuron_11: Tap,
    pub neuron_12: Tap,
    pub neuron_13: Tap,
    pub neuron_14: Tap,
    pub neuron_15: Tap,
    pub neuron_16: Tap,
    pub neuron_17: Tap,
    pub neuron_18: Tap,
    pub neuron_19: Tap,
    pub neuron_20: Tap,
    pub neuron_21: Tap,
    pub neuron_22: Tap,
    pub neuron_23: Tap,
}

impl RevenantTaps {
    /// The neurons the face shows, in network order.
    pub const NEURONS: [TapId; 24] = [
        Self::NEURON_0,
        Self::NEURON_1,
        Self::NEURON_2,
        Self::NEURON_3,
        Self::NEURON_4,
        Self::NEURON_5,
        Self::NEURON_6,
        Self::NEURON_7,
        Self::NEURON_8,
        Self::NEURON_9,
        Self::NEURON_10,
        Self::NEURON_11,
        Self::NEURON_12,
        Self::NEURON_13,
        Self::NEURON_14,
        Self::NEURON_15,
        Self::NEURON_16,
        Self::NEURON_17,
        Self::NEURON_18,
        Self::NEURON_19,
        Self::NEURON_20,
        Self::NEURON_21,
        Self::NEURON_22,
        Self::NEURON_23,
    ];

    fn neurons(&self) -> [&Tap; 24] {
        [
            &self.neuron_0,
            &self.neuron_1,
            &self.neuron_2,
            &self.neuron_3,
            &self.neuron_4,
            &self.neuron_5,
            &self.neuron_6,
            &self.neuron_7,
            &self.neuron_8,
            &self.neuron_9,
            &self.neuron_10,
            &self.neuron_11,
            &self.neuron_12,
            &self.neuron_13,
            &self.neuron_14,
            &self.neuron_15,
            &self.neuron_16,
            &self.neuron_17,
            &self.neuron_18,
            &self.neuron_19,
            &self.neuron_20,
            &self.neuron_21,
            &self.neuron_22,
            &self.neuron_23,
        ]
    }

    pub(super) fn publish(&self, surprise: f32, brain: &Brain) {
        self.surprise.set(f64::from(surprise));
        for (i, tap) in self.neurons().into_iter().enumerate() {
            tap.set(f64::from(brain.neuron(i)));
        }
    }
}
