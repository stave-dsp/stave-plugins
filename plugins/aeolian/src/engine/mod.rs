//! The aeolian ENGINE — a sympathetic string harp. The engine is its
//! COMPONENTS; `process` below is the whole per-block dance:
//! * `detect.rs` — `Detector`: twelve pitch-class bands → each
//!   string's excitation, gated by WIND (velocity) and dominance.
//! * `strings.rs` — `Strings`: twelve doubled Karplus–Strong voices,
//!   SUSTAIN = decay, TONE = damping, VIBRATO = per-voice drift.
//! * `ensemble.rs` — `Ensemble`: CHORUS width.
//! * `telemetry.rs` — `AeolianTaps`: each string's ring level and the
//!   harp's clock → the face's vibrating strings.
//!
//! Stereo: the twelve voices are spread around the circle of fifths,
//! so a chord fans out across the image.

use stave::prelude::*;

use crate::params::AeolianParams;

mod detect;
mod ensemble;
mod strings;
mod telemetry;

use self::detect::{Detector, CLASSES};
use self::ensemble::Ensemble;
use self::strings::{Strings, Voicing};
pub use self::telemetry::AeolianTaps;

/// The REGISTER selector's first position (C2) puts the strings in octave 2.
const LOW_OCTAVE: i32 = 2;
/// Coefficients follow the gliding knobs every this many frames.
const CONTROL_FRAMES: usize = 32;
/// The harp's level into the mix.
const WET_GAIN: f32 = 0.45;
/// The wet's top end: TONE sweeps this low-pass.
const TONE_LO_HZ: f32 = 500.0;
const TONE_HI_HZ: f32 = 14_000.0;
/// The wet low-pass's Q: a touch under Butterworth (k = 1.8).
const TONE_Q: f32 = 1.0 / 1.8;

/// Each voice's (left, right) gains: spread on the circle of fifths.
fn pans() -> [[f32; 2]; CLASSES] {
    core::array::from_fn(|c| {
        let pos = ((c * 7) % CLASSES) as f32 / (CLASSES - 1) as f32; // 0..1 around the fifths
        let p = (0.5 + 0.7 * (pos - 0.5)) * core::f32::consts::FRAC_PI_2;
        [p.cos(), p.sin()]
    })
}

#[derive(Smoothers)]
#[smoothers(params = AeolianParams)]
struct Controls {
    mix: Smoother,
    tone: Smoother,
    drift: Smoother,
    wind: Smoother,
    sustain: Smoother,
    output: Smoother,
}

pub struct Aeolian {
    params: Arc<AeolianParams>,
    controls: Controls,
    sample_rate: f32,
    detector: Detector,
    strings: Strings,
    ensemble: Ensemble,
    tone: [Svf; 2],
    pans: [[f32; 2]; CLASSES],
    taps: AeolianTaps,
    /// Seconds of audio played (wrapped) — the strings' animation clock.
    clock: f64,
}

impl Plugin for Aeolian {
    const INFO: Info = Info {
        id: "com.ryanbahan.stave.aeolian.vst3",
        name: "Stave Aeolian",
        vendor: "Ryan Bahan",
        url: "https://github.com/stave-dsp/stave",
        description: "A sympathetic string harp: twelve voices that ring with what you play.",
        features: &[Feature::AudioEffect, Feature::Filter, Feature::Stereo],
        ..Info::EFFECT
    };
    type Params = AeolianParams;

    fn new(params: &Arc<AeolianParams>, telemetry: &Telemetry) -> Self {
        Self {
            controls: Controls::new(params),
            params: params.clone(),
            sample_rate: 48_000.0,
            detector: Detector::new(),
            strings: Strings::new(),
            ensemble: Ensemble::new(),
            tone: [Svf::new(TONE_HI_HZ, TONE_Q); 2],
            pans: pans(),
            taps: AeolianTaps::new(telemetry),
            clock: 0.0,
        }
    }

    fn prepare(&mut self, env: &Environment) {
        self.sample_rate = env.sample_rate;
        self.controls.prepare(env);
        self.detector.prepare(env.sample_rate);
        self.strings.prepare(env.sample_rate);
        self.ensemble.prepare(env.sample_rate);
        self.ensemble.reset_to(self.params.air.value());
        for lp in &mut self.tone {
            lp.prepare(env.sample_rate);
        }
    }

    fn process(&mut self, block: &mut Block<'_>) {
        let p = &self.params;
        // 1. Bind: the register and the switches.
        let octave = LOW_OCTAVE + p.register.value().0 as i32;
        let (chorus, vibrato) = (p.air.value(), p.sway.value());
        let nyq = 0.45 * self.sample_rate;

        // 2. The audio block.
        let ControlsRamps {
            mix: mut mix_ramp,
            tone: mut tone_ramp,
            drift: mut mod_ramp,
            wind: mut sens_ramp,
            sustain: mut fb_ramp,
            output: mut out_ramp,
        } = self.controls.ramp(block, p);
        let (detector, strings, ensemble, tone, pans) =
            (&mut self.detector, &mut self.strings, &mut self.ensemble, &mut self.tone, &self.pans);
        let mut frame = 0usize;

        let lanes = block.lanes::<2>();
        let active = lanes.active();
        lanes.run(|fr| {
            let (t, m, s, fb) = (tone_ramp.next(), mod_ramp.next(), sens_ramp.next(), fb_ramp.next());
            if frame.is_multiple_of(CONTROL_FRAMES) {
                strings.voice(Voicing {
                    octave,
                    sustain: fb,
                    tone: t,
                    wind: s,
                    vibrato: if vibrato { m } else { 0.0 },
                });
                let hz = (TONE_LO_HZ * (TONE_HI_HZ / TONE_LO_HZ).powf(t)).min(nyq);
                for lp in tone.iter_mut() {
                    lp.set(hz, TONE_Q);
                }
            }
            frame += 1;

            let dry = if active >= 2 { [fr.input(0), fr.input(1)] } else { [fr.input(0); 2] };
            let mut exc = [0.0f32; CLASSES];
            detector.tick(0.5 * (dry[0] + dry[1]), s, &mut exc);
            let mut ring = [0.0f32; CLASSES];
            strings.tick(&exc, &mut ring);
            let mut harp = [0.0f32; 2];
            for (v, g) in ring.iter().zip(pans.iter()) {
                harp[0] += v * g[0];
                harp[1] += v * g[1];
            }
            let w = ensemble.tick([harp[0] * WET_GAIN, harp[1] * WET_GAIN], chorus, m);
            let w = [tone[0].tick(w[0]), tone[1].tick(w[1])];

            let a = mix_ramp.next() * core::f32::consts::FRAC_PI_2;
            // MIX 0 is bit-exact: cos(0) = 1 exactly, sin(0) = 0 exactly.
            let (dg, wg) = (a.cos(), a.sin());
            let g = db_to_amp(out_ramp.next());
            if active >= 2 {
                fr.write(0, (dry[0] * dg + w[0] * wg) * g);
                fr.write(1, (dry[1] * dg + w[1] * wg) * g);
            } else if active == 1 {
                fr.write(0, (dry[0] * dg + 0.5 * (w[0] + w[1]) * wg) * g);
            }
        });

        // 3. What happened, reported out.
        self.clock = (self.clock + block.frames() as f64 / f64::from(self.sample_rate)) % 3600.0;
        self.taps.publish(self.strings.levels(), self.clock);
    }

    /// The loops ring for up to their T60; at the top they sustain.
    fn tail(&self) -> Tail {
        let p = &self.params;
        if p.sustain.value() > 0.97 {
            Tail::Infinite
        } else {
            Tail::Samples((40.0 * self.sample_rate) as u32)
        }
    }

    fn face(params: &AeolianParams) -> Option<Face> {
        Some(crate::editor::face(params))
    }

    fn presets(params: &AeolianParams) -> Vec<Preset> {
        crate::presets::presets(params)
    }
}

