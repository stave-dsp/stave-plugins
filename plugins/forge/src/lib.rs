//! Stave Forge — an amp-topology saturator. **Start here**: this
//! file is the plugin's root; everything below is one click away.
//!
//! ```text
//! lib.rs        ← you are here: the modules and the export
//! params.rs     every knob, with its forever-stable id
//! editor.rs     the face, a pure function of params
//! views.rs      the face's own screens (transfer, harmonics, stages)
//! presets.rs    factory presets, pure function of params
//! engine/          the ENGINE half — the realtime processor
//!   mod.rs         `Forge` + its `impl Plugin`: identity, layouts,
//!                  and `process`, the per-block orchestration in
//!                  three named phases
//!   strip.rs       `Strip` — one M/S lane: topology + 4× region
//!   bindings.rs    `Bindings::apply_changes` — params → structure
//!   modulation.rs  `Modulation` — drivers, routes, displacement
//!   process.rs     `process_block` — the audio block
//!   telemetry.rs   `Observer::publish` — taps, capture, editor feeds
//!   voicing.rs     the sound-design vocabulary (curve recipes)
//! tests/        this project's properties: voicing suites, taps,
//!               the face, and every generic check (checks.rs)
//! ```
//!
//! What it sounds like — the breadth axes, exposed as DISCRETE
//! selectors (proper composition, not one morph knob):
//!
//! * CHARACTER — the harmonic recipe of the primary nonlinearity. Six
//!   ladders with genuinely different even/odd content and falloff
//!   (Chebyshev weight sets, knee families, and blends). EDGE morphs
//!   WITHIN the chosen family.
//! * STACK — the circuit topology. Single stage, two-stage cascade,
//!   screamer (a real feedback-clipper RC first — threshold rides on
//!   charge history), or push-pull (opposed-bias pair: crossover-ish
//!   odd emphasis).
//! * FILTER / POST FILTER — two explicit filter selectors. FILTER sits
//!   BETWEEN the stages (flat/scoop/bump/bright/dark/honk placements —
//!   where the G(f, level) bend comes from; TONE trims it). POST FILTER
//!   shapes the output (flat/air/low cut/warm/hi cut/dark), named as
//!   the EQ moves they are.
//! * MEMORY — the program-dependent behaviour: none, supply sag
//!   (drive give + darkening), grid blocking (sense-driven bias choke),
//!   squeeze (envelope gain compression), or ripple (env-scaled 120 Hz
//!   ghost sidebands). DEPTH scales it.
//! * SUB — the SubOctave divider, both roles: SUB MIX feeds the locked
//!   octave-down into the chain input (saturated by the same circuit);
//!   SUB MOD gain-modulates the primary stage at f0/2, putting energy
//!   at 1.5·f0/2.5·f0 — between the harmonics.
//!
//! Everything runs inside the nested 4× FIR stack; the chain is
//! prepared at 4× the host rate (rate contract); latency is the
//! constant 84 samples with a compensated dry path. M/S: the mid chain
//! always gets full drive; WIDTH scales the side chain.

mod editor;
mod engine;
mod params;
mod presets;
mod views;

pub use engine::{Forge, ForgeTaps};
pub use params::ForgeParams;

stave::export!(Forge, vst3_class_ids: {
    component: [0x5A7FE101, 0x6E5E4F88, 0x9D6D7E11, 0x55E82001],
    controller: [0x5A7FE101, 0x6E5E4F88, 0x9D6D7E11, 0x55E82002],
});
