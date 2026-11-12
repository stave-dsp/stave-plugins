//! Output conditioning after the amplifier: fixed voicing,
//! presence/depth, transformer bandwidth, and DC rejection.

use stave::prelude::*;

use crate::controls::OutputSettings;

pub struct OutputConditioner {
    fixed_low_mid_peak: Biquad,
    fixed_high_shelf: Biquad,
    presence_high_shelf: Biquad,
    depth_low_shelf: Biquad,
    first_high_frequency_lowpass: Biquad,
    second_high_frequency_lowpass: Biquad,
    direct_current_highpass: Biquad,
}

impl OutputConditioner {
    pub fn new() -> Self {
        Self {
            fixed_low_mid_peak: Biquad::passthrough(),
            fixed_high_shelf: Biquad::passthrough(),
            presence_high_shelf: Biquad::passthrough(),
            depth_low_shelf: Biquad::passthrough(),
            first_high_frequency_lowpass: Biquad::passthrough(),
            second_high_frequency_lowpass: Biquad::passthrough(),
            direct_current_highpass: Biquad::passthrough(),
        }
    }

    pub fn prepare(&mut self, sample_rate: f32) {
        let sample_rate = sample_rate.max(1.0);
        for filter in [
            &mut self.fixed_low_mid_peak,
            &mut self.fixed_high_shelf,
            &mut self.presence_high_shelf,
            &mut self.depth_low_shelf,
            &mut self.first_high_frequency_lowpass,
            &mut self.second_high_frequency_lowpass,
            &mut self.direct_current_highpass,
        ] {
            filter.prepare(sample_rate);
        }
        self.first_high_frequency_lowpass.set(FilterSpec::Lowpass {
                hz: 10_400.0,
                q: 0.6,
            });
        self.second_high_frequency_lowpass.set(FilterSpec::Lowpass {
                hz: 9_500.0,
                q: 0.55,
            });
        self.fixed_high_shelf.set(FilterSpec::HighShelfRbj {
                hz: 1_400.0,
                db: 8.5,
            });
        self.fixed_low_mid_peak.set(FilterSpec::Peak {
                hz: 330.0,
                db: 2.5,
                q: 0.9,
            });
        self.direct_current_highpass
            .set(FilterSpec::Highpass { hz: 10.0, q: 0.71 });
    }

    pub fn reset(&mut self) {
        for filter in [
            &mut self.fixed_low_mid_peak,
            &mut self.fixed_high_shelf,
            &mut self.presence_high_shelf,
            &mut self.depth_low_shelf,
            &mut self.first_high_frequency_lowpass,
            &mut self.second_high_frequency_lowpass,
            &mut self.direct_current_highpass,
        ] {
            filter.reset();
        }
    }

    pub fn apply_settings(&mut self, settings: OutputSettings) {
        self.presence_high_shelf.set(FilterSpec::HighShelfRbj {
                hz: 4_500.0,
                db: settings.presence_gain_db,
            });
        self.depth_low_shelf.set(FilterSpec::LowShelfRbj {
                hz: 90.0,
                db: settings.depth_gain_db,
            });
    }

    #[inline]
    pub fn process_sample(&mut self, input_sample: f32) -> f32 {
        let low_mid_shaped = self.fixed_low_mid_peak.tick(input_sample);
        let fixed_response = self.fixed_high_shelf.tick(low_mid_shaped);
        let presence_shaped = self.presence_high_shelf.tick(fixed_response);
        let depth_shaped = self.depth_low_shelf.tick(presence_shaped);
        let first_band_limited = self.first_high_frequency_lowpass.tick(depth_shaped);
        let second_band_limited = self
            .second_high_frequency_lowpass
            .tick(first_band_limited);
        self.direct_current_highpass.tick(second_band_limited)
    }
}
