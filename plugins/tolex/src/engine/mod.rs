//! The tolex ENGINE — a cabinet in three components per lane:
//!
//! ```text
//! mod.rs      ← you are here: the orchestration (bind → render)
//! motor.rs    Motor — the lumped electro-mechanical driver core
//!               (leaf: 3 f64 states, trapezoidal semi-implicit,
//!                Bl(x)/Cms(x) large-signal terms, k_box air spring)
//! cabinet.rs  RearPath — the open back's wrap-around radiator
//!               (delay + diffraction lowpass + mic-distance mix)
//! voice.rs    Voice — the cone's modal fingerprint: the fixed
//!               mode set (T(f) ring law Qs), the position system
//!               (weights + comb + macro walk), drive softening,
//!               and the piston cliff
//! ```
//!
//! Bind: the physical build sheet (size/fs/Qts/damping/back/cab)
//! rebuilds on a Watch, block rate. Render: per sample, per lane —
//! program swings the mesh at `drive` volts, the motor answers with
//! far-field pressure, the back door combs it, the voice rolls it.

use stave::params::Watch;
use stave::prelude::*;

pub(crate) mod cabinet;
pub(crate) mod drivers;
pub(crate) mod interior;
pub(crate) mod microphone;
pub(crate) mod motor;
pub(crate) mod voice;

use cabinet::RearPath;
use drivers::Drivers;
use interior::Interior;
use microphone::Microphone;
use motor::Motor;
use voice::Voice;

use crate::params::TolexParams;
use crate::{editor, presets};

/// The drive knob's 0 dB reference swing, volts at the mesh.
const V_REF: f32 = 8.0;

/// How often the voice reads the motor's excursion (samples). The
/// envelope itself lags ~10 ms; this just paces the coefficient
/// refresh.
const SOFTEN_INTERVAL: u32 = 64;

/// One audio lane: motor → back door → the voice.
struct Lane {
    motor: Motor,
    chorus: Drivers,
    back: RearPath,
    interior: Interior,
    voice: Voice,
    mic: Microphone,
}

/// The motor's live operating point, out to the face: the response
/// screen and the travel strip.
#[derive(Taps)]
pub struct TolexTaps {
    /// The motor's work (the squared, smoothed excursion fraction).
    pub excursion: Tap,
    /// |x| as a fraction of the Bl-droop scale.
    pub travel: Tap,
}

pub struct Tolex {
    params: Arc<TolexParams>,
    drive: Smoother,
    mic: Smoother,
    output: Smoother,
    sample_rate: f32,
    lanes: PerChannel<Lane, 2>,
    /// Rebuild-on-change for the physical build sheet. Reset at
    /// prepare: rate-derived coefficients are stale state the
    /// params can't see.
    build: Watch<15>,
    soften_phase: u32,
    taps: TolexTaps,
    /// True between prepare and the first bind — structure snaps
    /// instead of gliding (no history to protect yet).
    fresh: bool,
}

impl Tolex {
    /// Bind — params become structure, block rate, only on change.
    fn bind(&mut self) {
        let p = &self.params;
        let size = p.size.value().0 as usize;
        let fs = p.resonance.value();
        let qts = p.q.value();
        let d = p.damping.value();
        let rsrc = motor::rsrc_ohms(d);
        let open = p.back.value().0 == 1;
        let vb_l = p.cab.value();
        let vb_box = if open { None } else { Some(vb_l * 1e-3) };
        let mass = p.mass.value();
        let variant = p.cone.value().round().max(0.0) as usize;
        let material = p.material.value().0 as usize;
        let model = p.model.value().0 as usize;
        let mic_r = p.mic.value() * 0.01;
        let layout = p.speakers.value().0 as usize;
        let (breakup, doping, position) = (p.breakup.value(), p.doping.value(), p.position.value());
        let sr = self.sample_rate;
        for lane in self.lanes.iter_mut() {
            lane.motor.set(size, fs, qts, rsrc, vb_box, mass);
            lane.chorus.set(layout);
            lane.back.set(open, vb_l, sr);
            lane.interior.set(vb_l, material, open, layout, sr);
            lane.voice
                .set(size, variant, mass, breakup, doping, position);
            lane.mic.set(model, mic_r);
            if self.fresh {
                lane.back.snap();
            }
        }
        self.fresh = false;
    }
}

impl Plugin for Tolex {
    const INFO: Info = Info {
        id: "com.ryanbahan.stave.tolex.vst3",
        name: "Stave Tolex",
        vendor: "Ryan Bahan",
        url: "https://github.com/stave-dsp/stave",
        description: "A guitar cabinet, built not sampled.",
        features: &[Feature::AudioEffect, Feature::Stereo],
        ..Info::EFFECT
    };
    type Params = TolexParams;

    fn new(params: &Arc<TolexParams>, telemetry: &Telemetry) -> Self {
        Self {
            params: params.clone(),
            drive: Smoother::new(&params.drive),
            mic: Smoother::new(&params.mic),
            output: Smoother::new(&params.output),
            sample_rate: 48_000.0,
            lanes: PerChannel::new(|_| Lane {
                motor: Motor::new(),
                chorus: Drivers::new(),
                back: RearPath::new(),
                interior: Interior::new(),
                voice: Voice::new(),
                mic: Microphone::new(),
            }),
            build: Watch::new(),
            soften_phase: 0,
            fresh: true,
            taps: TolexTaps::new(telemetry),
        }
    }

    fn prepare(&mut self, env: &Environment) {
        self.drive.prepare(env);
        self.mic.prepare(env);
        self.output.prepare(env);
        self.sample_rate = env.sample_rate;
        for lane in self.lanes.iter_mut() {
            lane.motor.prepare(env.sample_rate);
            lane.chorus.prepare(env.sample_rate);
            lane.back.prepare(env.sample_rate);
            lane.interior.prepare(env.sample_rate);
            lane.voice.prepare(env.sample_rate);
            lane.mic.prepare(env.sample_rate);
        }
        self.build.reset();
        self.fresh = true;
    }

    fn process(&mut self, block: &mut Block<'_>) {
        let p = self.params.clone();
        if self.build.changed([
            &p.size,
            &p.resonance,
            &p.q,
            &p.damping,
            &p.back,
            &p.cab,
            &p.material,
            &p.model,
            &p.speakers,
            &p.mass,
            &p.breakup,
            &p.doping,
            &p.position,
            &p.mic,
            &p.cone,
        ]) {
            self.bind();
        }

        let mut drive = self.drive.ramp(block, &p.drive);
        let mut mic = self.mic.ramp(block, &p.mic);
        let mut output = self.output.ramp(block, &p.output);
        let lanes_state = &mut self.lanes;
        let lanes = block.lanes::<2>();
        let active = lanes.active();
        let mut soften_phase = self.soften_phase;
        lanes.run(|frame| {
            let vd = V_REF * db_to_amp(drive.next());
            let mic_r = mic.next() * 0.01; // cm → m
            let g = db_to_amp(output.next());
            let soften_now = soften_phase == 0;
            soften_phase = (soften_phase + 1) % SOFTEN_INTERVAL;
            for lane in 0..frame.lanes() {
                let s = &mut lanes_state[lane];
                let pressure = s.chorus.tick(s.motor.tick(frame.input(lane), vd), mic_r);
                if soften_now {
                    s.voice.soften(s.motor.work());
                }
                let at_mic = s.interior.tick(s.back.tick(pressure, mic_r));
                frame.write(lane, s.mic.tick(s.voice.tick(at_mic)) * g);
            }
        });
        self.soften_phase = soften_phase;

        // Observe: the block's operating point (max across lanes).
        let mut work = 0.0f32;
        let mut travel = 0.0f32;
        for lane in lanes_state.iter_mut().take(active) {
            work = work.max(lane.motor.work());
            travel = travel.max(lane.motor.excursion());
        }
        self.taps.excursion.set(f64::from(work));
        // Travel in its PHYSICAL unit: |x| as a fraction of the
        // Bl-droop scale (env is the squared, smoothed fraction —
        // sqrt puts normal playing at a visible few percent
        // instead of an invisible squared sliver).
        self.taps
            .travel
            .set(f64::from(travel.max(0.0).sqrt().clamp(0.0, 1.0)));
    }

    fn tail(&self) -> Tail {
        // The motor rings at its resonance (Q up to ~6 under tube
        // damping) and the wrap path adds ~2 ms — a quarter second
        // covers the worst alignment with margin.
        Tail::Samples((self.sample_rate * 0.25) as u32)
    }

    fn face(params: &TolexParams) -> Option<Face> {
        Some(editor::face(params))
    }

    fn presets(params: &TolexParams) -> Vec<Preset> {
        presets::presets(params)
    }
}
