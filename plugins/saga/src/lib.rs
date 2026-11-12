//! Stave Saga — a stereo reverse/reverb soundscape generator: two
//! time machines in series, each with its own feedback path, on a
//! 32 kHz clock you can pull. Modelled on the Walrus Audio Lore
//! (MKII) — docs/saga.md maps every pedal control to its param.
//! **Start here**: this file is the plugin's root; everything below
//! is one click away.
//!
//! ```text
//! lib.rs      ← you are here: the modules and the export
//! params.rs   every knob, with its forever-stable id
//! presets.rs  the pedal's RED / GREEN / BLUE + postcards
//! editor.rs   the face, a pure function of params
//! engine/     the ENGINE half:
//!   mod.rs        Saga: identity + the spine (bind → render → observe)
//!   motion.rs     Motion — the clock: base, DIVE/RISE, MOD LFO
//!   clock.rs      ClockDomain — host rate ⇄ the core clock
//!   programs.rs   Programs — the five wirings of the cells:
//!   reverse.rs    ReverseLine — a line read backward
//!   tank.rs       Tank — the 8-line FDN reverb
//!   transpose.rs  Transposer — the two-head pitch shifter
//!   path.rs       AnalogPath — the feedback loops' analog legs
//!   wet.rs        Wet — TONE, WIDTH, the soft ceiling
//!   telemetry.rs  SagaTaps — clock + tail out
//! tests/      the shared checks + THE PROPERTIES (properties.rs: bit-dry mix,
//!               reversal is real, octaves land, the dive drops an
//!               octave, programs are distinct, maxed loops stay
//!               bounded)
//! ```

mod editor;
mod engine;
mod params;
mod presets;

pub use engine::{Saga, SagaTaps};
pub use params::SagaParams;

stave::export!(Saga, vst3_class_ids: {
    component: [0xC0765B4B, 0x46762B10, 0x8645B824, 0x889E2C27],
    controller: [0x039D76AD, 0x9D871B51, 0x582038D9, 0xEA200920],
});
