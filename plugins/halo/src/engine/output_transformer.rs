//! Halo's product-specific output-transformer assembly.

use stave::dsp::FilterSpec;
use stave::dsp::{Biquad, Hysteresis, Stage};

#[derive(Clone)]
pub struct OutputTransformer {
    low_frequency_bandwidth: Biquad,
    magnetization: Hysteresis,
    high_frequency_bandwidth: Biquad,
}

impl OutputTransformer {
    pub fn new() -> Self {
        Self {
            low_frequency_bandwidth: Biquad::default(),
            magnetization: Hysteresis::new(0.2, 0.02),
            high_frequency_bandwidth: Biquad::default(),
        }
    }

    /// Clear every stage's running state, settings kept.
    pub fn reset(&mut self) {
        self.low_frequency_bandwidth.reset();
        self.magnetization.reset();
        self.high_frequency_bandwidth.reset();
    }

    pub fn prepare(&mut self, sample_rate: f32) {
        self.low_frequency_bandwidth.prepare(sample_rate);
        self.low_frequency_bandwidth
            .set(FilterSpec::Highpass { hz: 11.0, q: 0.62 });
        self.magnetization.prepare(sample_rate);
        self.high_frequency_bandwidth.prepare(sample_rate);
        self.high_frequency_bandwidth.set(FilterSpec::Lowpass {
            hz: 21_500.0,
            q: 0.71,
        });
    }

    #[inline]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let low_frequency_limited = self.low_frequency_bandwidth.tick(input);
        let magnetized = self.magnetization.tick(low_frequency_limited);
        self.high_frequency_bandwidth.tick(magnetized)
    }
}
