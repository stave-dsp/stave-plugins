//! Shared fixtures for the conformance suites.

use stave_dsp::{Oversampled2x, SmoothSat, Stage};

/// SmoothSat's memoryless shape as the interior of an oversampled region.
/// `prepare` is deliberately the no-op default: the companding filters
/// live in the SmoothSat and stay at the base rate; only the shape runs
/// inside the region, and it has no rate-derived state.
struct Shape(SmoothSat);

impl Stage for Shape {
    #[inline]
    fn tick(&mut self, x: f32) -> f32 {
        self.0.shape(x)
    }
}

/// Velvet's engine: SmoothSat's base-rate pre-emphasis, its shape through
/// the nested 4× FIR stack (`front(nested(..))`), then its base-rate
/// re-emphasis and DC block.
pub struct Velvet4x {
    region: Oversampled2x<Oversampled2x<Shape>>,
}

impl Velvet4x {
    pub fn new() -> Self {
        Self {
            region: Oversampled2x::front(Oversampled2x::nested(Shape(SmoothSat::new()))),
        }
    }

    /// The SmoothSat whose recipe the chain runs (setters go here).
    pub fn sat(&mut self) -> &mut SmoothSat {
        &mut self.region.inner.inner.0
    }
}

impl Stage for Velvet4x {
    fn prepare(&mut self, sample_rate: f32) {
        self.sat().prepare(sample_rate);
        self.region.prepare(sample_rate);
    }

    #[inline]
    fn tick(&mut self, x: f32) -> f32 {
        let pre = self.sat().tick_pre(x);
        let shaped = self.region.tick(pre);
        self.sat().tick_post(shaped)
    }
}
