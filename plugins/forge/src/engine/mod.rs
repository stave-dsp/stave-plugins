//! The Forge plugin — every piece is a named kernel form or a declared
//! plane, composed. `Forge` and its `impl Plugin` live here; params,
//! face and presets are their own files.
//!
//! The map, form by form:
//! * Audio path = `Oversampled2x::front(Oversampled2x::nested(Chain<6>))` per
//!   lane — Region owns the rate contract and the 66+18 latency
//!   arithmetic; Chain is the Sequence structure whose interiors are
//!   sense endpoints.
//! * Lanes = `PerChannel<Strip>` in the M/S domain; the lift is the
//!   kernel adapter pair (`ms_encode`/`ms_decode`), width scales the
//!   side lane.
//! * Control = `ModMatrix` (a kernel `Router` instance): detectors +
//!   LFO + divider as Sources, routes as data, displace-from-base at
//!   every inlet. Param bindings are `Watch`/`ModalWatch` sets.
//! * Structure changes = `SwapPoint` benches (moves, never allocs);
//!   dry path = `Compensator` on the latency laws.
//! * Observation = declared `ForgeTaps` + the framework `Scope`
//!   capture; the topology strip derives from `Chain::topology()`.
//! * `process()` is bind → render → observe, three calls.
//!
//! Files are named for the interfaces they define — the types make
//! the composition obvious; the file names just say where to look:
//! * `strip.rs` — `Strip`, one M/S lane: topology + region + dry pad.
//! * `bindings.rs` — `Bindings::apply_changes`: params → structure.
//! * `modulation.rs` — `Modulation`: drivers, routes, displacement.
//! * `process.rs` — `process_block`, the audio block.
//! * `telemetry.rs` — `Observer::publish`: taps, capture, editor feeds.
//! * here — the plugin struct and `process`, the orchestration.

use std::time::Instant;

use stave::dsp::PerChannel;
use stave::prelude::*;

use crate::params::ForgeParams;

mod bindings;
mod modulation;
mod process;
mod strip;
mod telemetry;
pub(crate) mod voicing;

use bindings::Bindings;
use modulation::Modulation;
use strip::Strip;
pub use telemetry::ForgeTaps;
use telemetry::Observer;

/// The nested 4× region's round trip: 66 (front pair) + 18 (nested
/// pair, at the outer rate). Pinned by `region_latency_matches_the_pinned_const`.
const FORGE_LATENCY: u32 = 84;

/// The plugin is its COMPONENTS — read the fields, know the design;
/// `process` below is the whole per-block dance through them.
pub struct Forge {
    params: Arc<ForgeParams>,
    sample_rate: f32,
    /// The audio path: one Strip per M/S lane.
    strips: PerChannel<Strip>,
    /// Params bound to structure — watches + rebuild rules.
    bindings: Bindings,
    /// Drivers, routes, and the state they displace.
    modulation: Modulation,
    /// Everything reported out: taps, capture, editor feeds.
    observer: Observer,
    smoothers: process::Smoothers,
}

impl Plugin for Forge {
    const INFO: Info = Info {
        id: "com.ryanbahan.stave.forge.vst3",
        name: "Stave Forge",
        vendor: "Ryan Bahan",
        url: "https://github.com/stave-dsp/stave",
        description: "Amp-topology saturation: discrete characters, stacks, voices and memory.",
        features: &[Feature::AudioEffect, Feature::Distortion, Feature::Stereo],
        ..Info::EFFECT
    };
    /// Stereo FX, stated (the default is the same — the flagship
    /// declares its negotiation surface rather than inheriting it).
    const LAYOUTS: &'static [Layout] = &[Layout::STEREO];
    type Params = ForgeParams;

    fn new(params: &Arc<ForgeParams>, telemetry: &Telemetry) -> Self {
        Forge {
            params: params.clone(),
            sample_rate: 48_000.0,
            strips: PerChannel::new(|_| Strip::new()),
            bindings: Bindings::new(),
            modulation: Modulation::new(),
            observer: Observer::new(telemetry),
            smoothers: process::Smoothers::new(params),
        }
    }

    fn prepare(&mut self, env: &Environment) {
        self.sample_rate = env.sample_rate;
        for strip in self.strips.iter_mut() {
            // The Region form owns the rate contract: one call, the
            // chain lands prepared at 4×.
            strip.region.prepare(env.sample_rate);
            strip.dry.allocate(FORGE_LATENCY);
        }
        self.modulation.prepare(env.sample_rate);
        self.observer.prepare(env.max_block);
        self.smoothers.prepare(env);
        // Force full re-application after (re)preparation.
        self.bindings.reset();
    }

    /// The per-block orchestration — the whole plugin in three
    /// phases, each a named component call:
    fn process(&mut self, block: &mut Block<'_>) {
        let started = Instant::now();
        let p = &*self.params;
        // 1. Param changes become structure (only where knobs moved).
        self.bindings.apply_changes(
            p,
            &mut self.strips,
            &mut self.modulation,
            &mut self.observer,
        );
        // 2. The audio block: strips tick under the modulation plane.
        let (peak_in, peak_out) = process::process_block(
            block,
            p,
            &mut self.smoothers,
            &mut self.strips,
            &mut self.modulation,
            &mut self.observer,
        );
        // 3. What happened, reported out.
        self.observer.publish(
            p,
            self.sample_rate,
            telemetry::BlockReport {
                frames: block.frames(),
                peak_in,
                peak_out,
                started,
            },
        );
    }

    fn latency(&self) -> u32 {
        FORGE_LATENCY
    }

    fn face(params: &ForgeParams) -> Option<Face> {
        Some(crate::editor::face(params))
    }

    fn presets(params: &ForgeParams) -> Vec<Preset> {
        crate::presets::presets(params)
    }
}

#[cfg(test)]
mod topology_tests {
    use super::strip::AmpChain;
    use super::*;

    /// law tie: the Region form computes the pinned latency.
    #[test]
    fn region_latency_matches_the_pinned_const() {
        let strip = Strip::new();
        assert_eq!(strip.region.latency(), FORGE_LATENCY);
    }

    /// Sidecar snapshot: the emitted topology data IS the built
    /// structure — this pin can only move when build_chain moves.
    #[test]
    fn declared_topology_matches_the_build() {
        let amp = AmpChain::new();
        let got: Vec<_> = amp.as_chain().topology().collect();
        assert_eq!(
            got,
            vec![
                (amp.primary, "curve"),
                (amp.voicing, "filter"),
                (amp.second, "curve"),
                (amp.post, "filter"),
                (amp.grit, "slew"),
                (amp.iron, "hysteresis"),
            ]
        );
    }
}
