use stave::dsp::FilterSpec;
use stave::dsp::{Biquad, Stage};

use crate::controls::InputSettings;

#[derive(Clone)]
pub struct InputConditioner {
    subsonic_highpass: Biquad,
    radio_frequency_lowpass: Biquad,
}

impl InputConditioner {
    pub fn new() -> Self {
        Self {
            subsonic_highpass: Biquad::default(),
            radio_frequency_lowpass: Biquad::default(),
        }
    }

    /// Clear every stage's running state, settings kept.
    pub fn reset(&mut self) {
        self.subsonic_highpass.reset();
        self.radio_frequency_lowpass.reset();
    }

    pub fn prepare(&mut self, sample_rate: f32) {
        let sample_rate = sample_rate.max(1.0);
        self.subsonic_highpass.prepare(sample_rate);
        self.radio_frequency_lowpass.prepare(sample_rate);
    }

    pub fn apply_settings(&mut self, settings: InputSettings) {
        self.subsonic_highpass.set(FilterSpec::Highpass {
            hz: settings.highpass_cutoff_hz,
            q: core::f32::consts::FRAC_1_SQRT_2,
        });
        self.radio_frequency_lowpass.set(FilterSpec::Lowpass {
            hz: settings.lowpass_cutoff_hz,
            q: core::f32::consts::FRAC_1_SQRT_2,
        });
    }

    #[inline]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let highpassed = self.subsonic_highpass.tick(input);
        self.radio_frequency_lowpass.tick(highpassed)
    }
}
