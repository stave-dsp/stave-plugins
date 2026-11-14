//! The revenant ENGINE — a listening machine. The engine is its
//! COMPONENTS; `process` below is the whole dance:
//! * `brain.rs` — `Brain`: the echo-state network and its online
//!   readout (the only thing that learns).
//! * `telemetry.rs` — `RevenantTaps`: surprise and neuron activity out.
//!
//! The brain runs at ~12 kHz (it thinks in longer steps, so a period
//! of a played note spans enough of its memory): the host signal is
//! box-decimated in, the brain's guess is interpolated and smoothed
//! back out. What you hear at MIX 1 is ONLY its guess at your near
//! future — silent until it has learned something, ghostly after.
//!
//! ATTENTION: it speaks while you play. Stop (down to an interface's
//! noise floor) and its voice lets go over 1.5 s, however wound up
//! CHAOS and DREAM have it — a network past the edge would otherwise
//! hum on the hiss forever. Only SLEEP keeps it playing alone.

use stave::prelude::*;

use crate::params::RevenantParams;

mod brain;
mod telemetry;

use self::brain::{Brain, Mind};
pub use self::telemetry::RevenantTaps;

/// The brain's target rate.
const BRAIN_HZ: f32 = 12_000.0;
/// The smoothing on the brain's voice, back at host rate.
const VOICE_HZ: f32 = 4_800.0;
/// The DC blocker's corner on the voice.
const DC_HZ: f32 = 15.0;
/// SURPRISE's envelope time.
const SURPRISE_MS: f32 = 120.0;
/// The voice's soft ceiling.
const CEILING: f32 = 1.2;
/// ATTENTION: while your input's envelope is above this (≈ −54 dBFS
/// mean level — above any interface's hiss, below the quietest
/// playing) it speaks; below it, its voice fades over LET_GO_S, unless
/// SLEEP holds it. It returns within RETURN_S of you playing again.
const ATTENTION_GATE: f32 = 0.002;
const LET_GO_S: f32 = 1.5;
const RETURN_S: f32 = 0.03;

#[inline]
fn soft(x: f32) -> f32 {
    let x = (x / CEILING).clamp(-3.0, 3.0);
    CEILING * x * (27.0 + x * x) / (27.0 + 9.0 * x * x)
}

#[derive(Smoothers)]
#[smoothers(params = RevenantParams)]
struct Controls {
    learn: Smoother,
    memory: Smoother,
    chaos: Smoother,
    dream: Smoother,
    mix: Smoother,
    output: Smoother,
}

pub struct Revenant {
    params: Arc<RevenantParams>,
    controls: Controls,
    sample_rate: f32,
    brain: Brain,
    /// Host samples per brain step, and where we are in the step.
    decim: usize,
    phase: usize,
    acc: f32,
    /// The brain's last two guesses (interpolated between) and the
    /// voice's smoothing state.
    y0: f32,
    y1: f32,
    voice: OnePole,
    /// DC blocker on the voice.
    dc: DcBlocker,
    /// SURPRISE: envelopes of the error and of the input.
    env_err: f32,
    env_in: f32,
    env_k: f32,
    /// ATTENTION: the voice's gain (1 = it speaks), and its glides.
    attention: f32,
    let_go: f32,
    come_back: f32,
    taps: RevenantTaps,
}

impl Plugin for Revenant {
    const INFO: Info = Info {
        id: "com.ryanbahan.stave.revenant.vst3",
        name: "Stave Revenant",
        vendor: "Ryan Bahan",
        url: "https://github.com/stave-dsp/stave",
        description: "A neural network that learns your playing and keeps playing when you stop.",
        features: &[Feature::AudioEffect, Feature::Stereo],
        ..Info::EFFECT
    };
    type Params = RevenantParams;

    fn new(params: &Arc<RevenantParams>, telemetry: &Telemetry) -> Self {
        Self {
            controls: Controls::new(params),
            params: params.clone(),
            sample_rate: 48_000.0,
            brain: Brain::new(),
            decim: 4,
            phase: 0,
            acc: 0.0,
            y0: 0.0,
            y1: 0.0,
            voice: OnePole::lowpass(hz(VOICE_HZ)),
            dc: DcBlocker::new(hz(DC_HZ)),
            env_err: 0.0,
            env_in: 0.0,
            env_k: 0.01,
            attention: 0.0,
            let_go: 0.0,
            come_back: 0.0,
            taps: RevenantTaps::new(telemetry),
        }
    }

    fn prepare(&mut self, env: &Environment) {
        let sample_rate = env.sample_rate;
        self.sample_rate = sample_rate;
        self.controls.prepare(env);
        self.decim = ((sample_rate / BRAIN_HZ).round() as usize).max(1);
        self.voice.prepare(sample_rate);
        self.dc.prepare(sample_rate);
        let brain_hz = sample_rate / self.decim as f32;
        self.env_k = 1.0 - (-1000.0 / (SURPRISE_MS * brain_hz)).exp();
        self.reset();
        self.let_go = 1.0 - (-1.0 / (LET_GO_S * sample_rate / 4.6)).exp();
        self.come_back = 1.0 - (-1.0 / (RETURN_S * sample_rate)).exp();
    }

    /// Forget everything: the brain, its voice, its attention.
    fn reset(&mut self) {
        self.brain.reset();
        self.phase = 0;
        self.acc = 0.0;
        self.y0 = 0.0;
        self.y1 = 0.0;
        self.voice.reset();
        self.dc.reset();
        self.env_err = 0.0;
        self.env_in = 0.0;
        self.attention = 0.0;
    }

    fn process(&mut self, block: &mut Block<'_>) {
        let p = &self.params;
        let brain_hz = self.sample_rate / self.decim as f32;
        let horizon_steps = (p.horizon.value() * 0.001 * brain_hz).round() as usize;
        let asleep = p.sleep.value();

        let ControlsRamps {

            learn: mut learn_ramp,

            memory: mut memory_ramp,

            chaos: mut chaos_ramp,

            dream: mut dream_ramp,

            mix: mut mix_ramp,

            output: mut out_ramp,

        } = self.controls.ramp(block, p);
        let decim = self.decim;
        let inv = 1.0 / decim as f32;
        let (brain, phase, acc) = (&mut self.brain, &mut self.phase, &mut self.acc);
        let (y0, y1, voice_lp, dc) = (&mut self.y0, &mut self.y1, &mut self.voice, &mut self.dc);
        let (env_err, env_in, env_k) = (&mut self.env_err, &mut self.env_in, self.env_k);
        let (attention, let_go, come_back) = (&mut self.attention, self.let_go, self.come_back);

        let lanes = block.lanes::<2>();
        let active = lanes.active();
        lanes.run(|fr| {
            let (learn, memory, chaos, dream) = (learn_ramp.next(), memory_ramp.next(), chaos_ramp.next(), dream_ramp.next());
            let dry = if active >= 2 { [fr.input(0), fr.input(1)] } else { [fr.input(0); 2] };
            // Into the brain: a box average over its step.
            *acc += 0.5 * (dry[0] + dry[1]);
            *phase += 1;
            if *phase == decim {
                let u = *acc * inv;
                *acc = 0.0;
                *phase = 0;
                let m = Mind::from_knobs(learn, memory, chaos, horizon_steps, dream, asleep);
                let (y, err) = brain.tick(u, m);
                *y0 = *y1;
                *y1 = y;
                *env_err += (err.abs() - *env_err) * env_k;
                *env_in += (u.abs() - *env_in) * env_k;
            }
            // Out of the brain: interpolate its guesses, smooth.
            let t = *phase as f32 * inv;
            let raw = *y0 + (*y1 - *y0) * t;
            let blocked = dc.tick(voice_lp.tick(raw));
            // ATTENTION: it speaks while you play (or while it sleeps);
            // otherwise its voice lets go — the network still thinks.
            let engaged = asleep || *env_in > ATTENTION_GATE;
            if engaged {
                *attention += (1.0 - *attention) * come_back;
            } else {
                *attention += (0.0 - *attention) * let_go;
                if *attention < 1e-6 {
                    *attention = 0.0;
                }
            }
            let voice = soft(blocked) * *attention;

            let a = mix_ramp.next() * core::f32::consts::FRAC_PI_2;
            // MIX 0 is bit-exact: cos(0) = 1 exactly, sin(0) = 0 exactly.
            let (dg, wg) = (a.cos(), a.sin());
            let g = db_to_amp(out_ramp.next());
            if active >= 2 {
                fr.write(0, (dry[0] * dg + voice * wg) * g);
                fr.write(1, (dry[1] * dg + voice * wg) * g);
            } else if active == 1 {
                fr.write(0, (dry[0] * dg + voice * wg) * g);
            }
        });

        let surprise = if self.env_in > 1e-4 { (self.env_err / self.env_in).clamp(0.0, 1.0) } else { 0.0 };
        self.taps.publish(surprise, &self.brain);
    }

    /// A ringing or dreaming brain never has to stop.
    fn tail(&self) -> Tail {
        let p = &self.params;
        if p.chaos.value() > 0.5 || p.dream.value() > 0.0 || p.sleep.value() {
            Tail::Infinite
        } else {
            Tail::Samples((2.0 * self.sample_rate) as u32)
        }
    }

    fn face(params: &RevenantParams) -> Option<Face> {
        Some(crate::editor::face(params))
    }

    fn presets(params: &RevenantParams) -> Vec<Preset> {
        crate::presets::presets(params)
    }
}
