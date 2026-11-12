use stave::dsp::FilterSpec;
use stave::dsp::{Biquad, Stage};

#[derive(Clone)]
pub struct OutputConditioner {
    direct_current_highpass: Biquad,
    ultrasonic_lowpass: Biquad,
}

impl OutputConditioner {
    pub fn new() -> Self {
        Self {
            direct_current_highpass: Biquad::default(),
            ultrasonic_lowpass: Biquad::default(),
        }
    }

    /// Clear every stage's running state, settings kept.
    pub fn reset(&mut self) {
        self.direct_current_highpass.reset();
        self.ultrasonic_lowpass.reset();
    }

    pub fn prepare(&mut self, sample_rate: f32) {
        self.direct_current_highpass.prepare(sample_rate);
        self.direct_current_highpass
            .set(FilterSpec::Highpass { hz: 7.0, q: 0.71 });
        self.ultrasonic_lowpass.prepare(sample_rate);
        self.ultrasonic_lowpass.set(FilterSpec::Lowpass {
            hz: 22_000.0,
            q: 0.71,
        });
    }

    #[inline]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let direct_current_rejected = self.direct_current_highpass.tick(input);
        self.ultrasonic_lowpass.tick(direct_current_rejected)
    }
}
