//! `Wet` — the wet path's last stage, at the host rate: TONE (a
//! synth-style resonant low-pass — closed lets only lows through,
//! open passes the whole band), WIDTH (the image's side level: 0 =
//! mono, the middle = as the core made it, up = wider), and a soft
//! ceiling so a maxed feedback network saturates instead of clipping
//! the host.

use stave::dsp::{ms_decode, ms_encode, Stage, Svf};

use super::path::soft;

const TONE_LO_HZ: f32 = 120.0;
const TONE_HI_HZ: f32 = 18_000.0;
/// A touch past Butterworth: the synth filter's hint of a peak.
const TONE_Q: f32 = 1.0 / 1.4;
/// The ceiling's knee: below it the stage is linear.
const CEILING: f32 = 1.5;

pub(super) struct Wet {
    /// TONE, per lane.
    tone: [Svf; 2],
}

impl Wet {
    pub(super) fn new() -> Self {
        Self { tone: [Svf::default(); 2] }
    }

    pub(super) fn prepare(&mut self, host_hz: f32) {
        for f in &mut self.tone {
            f.prepare(host_hz);
        }
    }

    #[inline]
    pub(super) fn process(&mut self, x: [f32; 2], tone: f32, width: f32) -> [f32; 2] {
        // Per frame: TONE's glide is part of the instrument (a filter
        // sweep), so the corner follows the smoothed knob exactly.
        let hz = TONE_LO_HZ * (TONE_HI_HZ / TONE_LO_HZ).powf(tone);
        for f in &mut self.tone {
            f.set(hz, TONE_Q);
        }
        let l = self.tone[0].tick(x[0]);
        let r = self.tone[1].tick(x[1]);
        let (m, s) = ms_encode(l, r);
        let (l, r) = ms_decode(m, s * f64::from(2.0 * width));
        [ceiling(l), ceiling(r)]
    }
}

#[inline]
fn ceiling(x: f32) -> f32 {
    if x.abs() <= CEILING {
        x
    } else {
        let over = (x.abs() - CEILING) / CEILING;
        x.signum() * CEILING * (1.0 + soft(over) * 0.5)
    }
}
