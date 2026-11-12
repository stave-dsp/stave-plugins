//! `Spring` — a two-spring tank, the classic way it is modelled
//! (Abel/Välimäki's dispersive loop, simplified): each spring is a
//! delay loop whose path runs through a long chain of first-order
//! allpasses. The chain is DISPERSIVE — highs and lows travel at
//! different speeds — so every reflection arrives smeared into a
//! falling chirp: the spring's "drip". A low-pass in the loop is the
//! wire's damping; a high-pass is the transducer that cannot move air
//! below ~150 Hz. The two springs differ in length, so the tank is wide.

use stave::prelude::*;

const STAGES: usize = 48;
/// Allpass coefficient: how dispersive each stage is.
const DISPERSION: f32 = 0.62;
const LOOP_MS: [f32; 2] = [33.0, 41.0];
const FEEDBACK: f32 = 0.74;
const DAMP_HZ: f32 = 4_500.0;
const XDUCER_HZ: f32 = 150.0;
/// The damping and transducer filters' Q, a touch under Butterworth.
const Q: f32 = 1.0 / 1.8;

struct OneSpring {
    chain: [f32; STAGES],
    line: DelayLine,
    len: f32,
    fb: f32,
}

impl OneSpring {
    fn new() -> Self {
        Self { chain: [0.0; STAGES], line: DelayLine::new(), len: 1.0, fb: 0.0 }
    }

    #[inline]
    fn tick(&mut self, x: f32, damp: &mut Svf, xducer: &mut Svf) -> f32 {
        // Loop: input + damped feedback → dispersive chain → delay.
        let mut s = x + self.fb;
        for z in self.chain.iter_mut() {
            // First-order allpass, transposed: y = −a·s + z; z = s + a·y.
            let y = -DISPERSION * s + *z;
            *z = s + DISPERSION * y;
            s = y;
        }
        self.line.write(s);
        let back = self.line.read(self.len);
        self.line.advance();
        let damped = damp.tick(back);
        self.fb = FEEDBACK * damped;
        xducer.taps(damped).hp
    }
}

pub(super) struct Spring {
    springs: [OneSpring; 2],
    /// Per spring: the wire's damping and the transducer.
    damp: [Svf; 2],
    xducer: [Svf; 2],
}

impl Spring {
    pub(super) fn new() -> Self {
        Self { springs: [OneSpring::new(), OneSpring::new()], damp: [Svf::new(DAMP_HZ, Q); 2], xducer: [Svf::new(XDUCER_HZ, Q); 2] }
    }

    pub(super) fn prepare(&mut self, sample_rate: f32) {
        for (s, ms) in self.springs.iter_mut().zip(LOOP_MS) {
            s.len = ms * 0.001 * sample_rate;
            s.line.allocate(s.len as usize + 4);
            s.chain = [0.0; STAGES];
            s.fb = 0.0;
        }
        for (damp, xducer) in self.damp.iter_mut().zip(&mut self.xducer) {
            damp.prepare(sample_rate);
            damp.set(DAMP_HZ.min(0.45 * sample_rate), Q);
            xducer.prepare(sample_rate);
        }
    }

    /// One frame of mono drive → the tank's [L, R].
    #[inline]
    pub(super) fn tick(&mut self, x: f32) -> [f32; 2] {
        let l = self.springs[0].tick(x, &mut self.damp[0], &mut self.xducer[0]);
        let r = self.springs[1].tick(x, &mut self.damp[1], &mut self.xducer[1]);
        [l, r]
    }
}
