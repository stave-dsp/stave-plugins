//! The 3-band Linkwitz-Riley crossover splitter — one concept, two
//! instances: the MAIN split feeds the audio path, and the SIDECHAIN
//! split feeds detection from the external aux at the same
//! cutoffs with its own state.
//!
//! Not [`stave::dsp::Crossover3`]: that one phase-matches the low band
//! with an allpass at the high corner (a flat sum), this one does not —
//! swapping it would change every session's sound, so it stays until a
//! state-versioned change takes it.

use stave::dsp::{LinkwitzRiley4, Stage};
use stave::dsp::units::{hz, Hz};

pub(crate) struct Splitter {
    /// LP at low_xover → low band; HP at low_xover → "above low";
    /// then HP/LP at high_xover carve high/mid out of it. Per lane.
    lp_lo: [LinkwitzRiley4; 2],
    hp_lo: [LinkwitzRiley4; 2],
    lp_hi: [LinkwitzRiley4; 2],
    hp_hi: [LinkwitzRiley4; 2],
}

impl Splitter {
    pub(crate) fn new(low_hz: f32, high_hz: f32) -> Self {
        Self {
            lp_lo: [LinkwitzRiley4::lowpass(hz(low_hz)); 2],
            hp_lo: [LinkwitzRiley4::highpass(hz(low_hz)); 2],
            lp_hi: [LinkwitzRiley4::lowpass(hz(high_hz)); 2],
            hp_hi: [LinkwitzRiley4::highpass(hz(high_hz)); 2],
        }
    }

    fn filters(&mut self) -> impl Iterator<Item = &mut LinkwitzRiley4> {
        self.lp_lo
            .iter_mut()
            .chain(&mut self.hp_lo)
            .chain(&mut self.lp_hi)
            .chain(&mut self.hp_hi)
    }

    /// Coefficients at `sample_rate`; state cleared.
    pub(crate) fn prepare(&mut self, sample_rate: f32) {
        for f in self.filters() {
            f.prepare(sample_rate);
        }
    }

    pub(crate) fn reset(&mut self) {
        for f in self.filters() {
            f.reset();
        }
    }

    pub(crate) fn set_low(&mut self, hz: f32) {
        for f in self.lp_lo.iter_mut().chain(&mut self.hp_lo) {
            f.set_frequency(Hz::new(hz));
        }
    }

    pub(crate) fn set_high(&mut self, hz: f32) {
        for f in self.lp_hi.iter_mut().chain(&mut self.hp_hi) {
            f.set_frequency(Hz::new(hz));
        }
    }

    /// Split one frame into [(lo_l, lo_r), (mid_l, mid_r), (hi_l, hi_r)].
    /// `stereo = false` ticks only lane 0 and mirrors it — the mono
    /// path's lane-1 states never advance (bit-compat contract).
    #[inline]
    pub(crate) fn split(&mut self, l: f32, r: f32, stereo: bool) -> [(f32, f32); 3] {
        let lo_l = self.lp_lo[0].tick(l);
        let lo_r = if stereo { self.lp_lo[1].tick(r) } else { lo_l };
        let mid_pre_l = self.hp_lo[0].tick(l);
        let mid_pre_r = if stereo {
            self.hp_lo[1].tick(r)
        } else {
            mid_pre_l
        };
        let hi_l = self.hp_hi[0].tick(mid_pre_l);
        let hi_r = if stereo {
            self.hp_hi[1].tick(mid_pre_r)
        } else {
            hi_l
        };
        let mid_l = self.lp_hi[0].tick(mid_pre_l);
        let mid_r = if stereo {
            self.lp_hi[1].tick(mid_pre_r)
        } else {
            mid_l
        };
        [(lo_l, lo_r), (mid_l, mid_r), (hi_l, hi_r)]
    }
}
