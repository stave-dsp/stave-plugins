//! The smear ENGINE — the spine: the per-block process loop and the
//! control-rate updates that keep the field drifting. The garden
//! itself (the resonator lattice) is `garden.rs`.

use stave::params::Ramp;
use stave::prelude::*;

mod garden;
use garden::*;

use crate::params::SmearParams;

/// Declared observation points: the editor's IN/OUT meters
/// read these wires (dB peaks per block).
#[derive(Taps)]
pub struct SmearTaps {
    pub io_in: Tap,
    pub io_out: Tap,
}

/// One [`Smoother`] per knob: every control glides (the garden's at
/// 50 ms, the excitation and tone at 20 ms, the gains at 5 ms).
struct Controls {
    seed: Smoother,
    smear: Smoother,
    bloom: Smoother,
    decay: Smoother,
    flux: Smoother,
    shimmer: Smoother,
    drive: Smoother,
    motion: Smoother,
    depth: Smoother,
    tone: Smoother,
    mix: Smoother,
    output: Smoother,
}

/// One frame of the controls' values.
struct Values {
    seed: f32,
    smear: f32,
    bloom: f32,
    decay: f32,
    flux: f32,
    shimmer: f32,
    drive: f32,
    motion: f32,
    depth: f32,
    tone: f32,
    mix: f32,
    output: f32,
}

impl Controls {
    fn new(p: &SmearParams) -> Self {
        Self {
            seed: Smoother::new(&p.seed),
            smear: Smoother::new(&p.smear),
            bloom: Smoother::new(&p.bloom),
            decay: Smoother::new(&p.decay),
            flux: Smoother::new(&p.flux),
            shimmer: Smoother::new(&p.shimmer),
            drive: Smoother::new(&p.drive),
            motion: Smoother::new(&p.motion),
            depth: Smoother::new(&p.depth),
            tone: Smoother::new(&p.tone),
            mix: Smoother::new(&p.mix),
            output: Smoother::new(&p.output),
        }
    }

    fn prepare(&mut self, env: &Environment) {
        self.seed.prepare(env);
        self.smear.prepare(env);
        self.bloom.prepare(env);
        self.decay.prepare(env);
        self.flux.prepare(env);
        self.shimmer.prepare(env);
        self.drive.prepare(env);
        self.motion.prepare(env);
        self.depth.prepare(env);
        self.tone.prepare(env);
        self.mix.prepare(env);
        self.output.prepare(env);
    }
}

/// One block's ramps, one per knob.
struct Ramps<'s> {
    seed: Ramp<'s>,
    smear: Ramp<'s>,
    bloom: Ramp<'s>,
    decay: Ramp<'s>,
    flux: Ramp<'s>,
    shimmer: Ramp<'s>,
    drive: Ramp<'s>,
    motion: Ramp<'s>,
    depth: Ramp<'s>,
    tone: Ramp<'s>,
    mix: Ramp<'s>,
    output: Ramp<'s>,
}

impl Controls {
    fn ramps<'s>(&'s mut self, block: &Block<'s>, p: &SmearParams) -> Ramps<'s> {
        Ramps {
            seed: self.seed.ramp(block, &p.seed),
            smear: self.smear.ramp(block, &p.smear),
            bloom: self.bloom.ramp(block, &p.bloom),
            decay: self.decay.ramp(block, &p.decay),
            flux: self.flux.ramp(block, &p.flux),
            shimmer: self.shimmer.ramp(block, &p.shimmer),
            drive: self.drive.ramp(block, &p.drive),
            motion: self.motion.ramp(block, &p.motion),
            depth: self.depth.ramp(block, &p.depth),
            tone: self.tone.ramp(block, &p.tone),
            mix: self.mix.ramp(block, &p.mix),
            output: self.output.ramp(block, &p.output),
        }
    }
}

impl Ramps<'_> {
    fn next(&mut self) -> Values {
        Values {
            seed: self.seed.next(),
            smear: self.smear.next(),
            bloom: self.bloom.next(),
            decay: self.decay.next(),
            flux: self.flux.next(),
            shimmer: self.shimmer.next(),
            drive: self.drive.next(),
            motion: self.motion.next(),
            depth: self.depth.next(),
            tone: self.tone.next(),
            mix: self.mix.next(),
            output: self.output.next(),
        }
    }
}

pub struct Smear {
    sample_rate: f32,
    taps: SmearTaps,
    chans: PerChannel<ChanState, MAX_CHANS>,
    ctrl_phase: usize,

    controls: Controls,
    params: Arc<SmearParams>,
}

/// Cubic soft-clip: transparent below ~0.7, asymptotic toward ±1.
/// (Not the framework's `soft_clip`, which is tanh: this one's knee is
/// the garden's voice.)
#[inline]
fn soft_clip(x: f32) -> f32 {
    let t = (x * 0.7).clamp(-1.5, 1.5);
    (1.5 * t - 0.5 * t * t * t).clamp(-1.0, 1.0)
}

impl Plugin for Smear {
    const INFO: Info = Info {
        id: "com.ryanbahan.stave.smear.vst3",
        name: "Stave Smear",
        vendor: "Ryan Bahan",
        url: "https://github.com/stave-dsp/stave",
        description: "Sympathetic Garden — self-tuning resonator field; smear blooms across pitches.",
        features: &[Feature::AudioEffect, Feature::Reverb, Feature::Stereo],
        ..Info::EFFECT
    };
    /// The declared set — stereo preferred, 5.1 as the surround
    /// variant of the PerChannel lift. The gardens are per-lane
    /// independent, so the lift is exactly what lanes are for; the
    /// stereo pair is bit-pinned through it.
    const LAYOUTS: &'static [Layout] = &[
        Layout::STEREO,
        Layout { input: Some(Channels::Surround51), output: Channels::Surround51, aux: &[] },
    ];
    type Params = SmearParams;

    fn new(params: &Arc<SmearParams>, telemetry: &Telemetry) -> Self {
        Self {
            sample_rate: 48_000.0,
            taps: SmearTaps::new(telemetry),
            chans: PerChannel::new(|lane| {
                // Lanes 0/1 keep their historical seeds (stereo is
                // bit-pinned through the 5.1 lift); rears/C/LFE get
                // their own gardens.
                ChanState::new(
                    [
                        0x1234_5678,
                        0x9E37_79B9,
                        0x6A09_E667,
                        0xBB67_AE85,
                        0x3C6E_F372,
                        0xA54F_F53A,
                    ][lane],
                )
            }),
            ctrl_phase: 0,
            controls: Controls::new(params),
            params: params.clone(),
        }
    }

    fn prepare(&mut self, env: &Environment) {
        let sample_rate = env.sample_rate;
        self.sample_rate = sample_rate;
        for c in self.chans.iter_mut() {
            c.reset();
        }
        self.ctrl_phase = 0;
        self.controls.prepare(env);
        for c in self.chans.iter_mut() {
            c.update_coeffs(sample_rate, self.params.decay.value(), 0.0, 0.0);
        }
    }

    fn reset(&mut self) {
        for c in self.chans.iter_mut() {
            c.reset();
            c.update_coeffs(self.sample_rate, self.params.decay.value(), 0.0, 0.0);
        }
        self.ctrl_phase = 0;
    }

    fn process(&mut self, block: &mut Block<'_>) {
        let mut ramps = self.controls.ramps(block, &self.params);
        let sample_rate = self.sample_rate;
        let (gardens, ctrl_phase) = (&mut self.chans, &mut self.ctrl_phase);
        // Makeup gain: only a handful of resonators ring at once and each is
        // normalised to ~unity peak, so the raw sum is quiet. Bring the wet
        // up to roughly dry level; soft-clip catches dense excitation.
        let norm = 14.0 / (N_RES as f32).sqrt();

        let peaks = block.lanes::<MAX_CHANS>().run(|frame| {
            let v = ramps.next();
            let out_lin = db_to_amp(v.output);
            // Control-rate update: coefficients + tuning drift.
            if *ctrl_phase == 0 {
                let depth = v.depth;
                for (ci, c) in gardens.iter_mut().enumerate() {
                    c.update_coeffs(sample_rate, v.decay, v.motion, v.flux);
                    // DEPTH widens the image by detuning the gardens
                    // apart — a per-lane spread map (L/R the classic
                    // pair, rears wider, C/LFE anchored). The stereo
                    // entries reproduce the historical arithmetic
                    // bit-exactly (1 − x ≡ 1 + (−1)·x).
                    if depth > 0.0 {
                        const SPREAD: [f32; MAX_CHANS] = [-1.0, 1.0, 0.0, 0.0, -1.6, 1.6];
                        let widen = 1.0 + SPREAD[ci] * (depth * 0.004);
                        if SPREAD[ci] != 0.0 {
                            for d in c.detune.iter_mut() {
                                *d *= widen;
                            }
                        }
                    }
                }
            }
            *ctrl_phase = (*ctrl_phase + 1) % CTRL_INTERVAL;

            // Coupling/feedback gains, scaled so the network of
            // mutually-exciting resonators stays stable.
            let smear = v.smear * 0.35;
            let bloom = v.bloom * 0.5;
            let shimmer = v.shimmer * 0.4;
            let drive = v.drive;
            let drive_k = 1.0 + drive * 6.0;
            let exc_gain = v.seed * 0.6;

            for c in 0..frame.lanes() {
                let dry = frame.input(c);
                let g = &mut gardens[c];

                let mut wet = 0.0f32;
                for i in 0..N_RES {
                    // Sympathetic excitation: dry input + energy blooming in
                    // from neighbouring semitones (smear), an octave below
                    // (shimmer), and the resonator's own regeneration
                    // (bloom). Coupling reads the PREVIOUS sample's outputs.
                    let lo = if i > 0 { g.out_prev[i - 1] } else { 0.0 };
                    let hi = if i + 1 < N_RES {
                        g.out_prev[i + 1]
                    } else {
                        0.0
                    };
                    let oct = if i >= 12 { g.out_prev[i - 12] } else { 0.0 };
                    let mut exc =
                        dry * exc_gain + smear * (lo + hi) + shimmer * oct + bloom * g.out_prev[i];

                    // DRIVE — soft saturation in the excitation path adds
                    // warmth and tames the regenerative feedback.
                    if drive > 0.0 {
                        exc = (exc * drive_k).tanh() / drive_k.sqrt();
                    }

                    let y = g.a1[i] * g.y1[i] - g.a2[i] * g.y2[i] + g.b0[i] * exc;
                    let y = y + DENORM - DENORM; // denormal guard
                    g.y2[i] = g.y1[i];
                    g.y1[i] = y;
                    g.out_cur[i] = y;
                    wet += y;
                }
                core::mem::swap(&mut g.out_prev, &mut g.out_cur);

                let wet = soft_clip(wet * norm);

                // TONE: one-pole HF damping.
                let lp = 1.0 - v.tone * 0.9;
                g.tone_state = g.tone_state * (1.0 - lp) + wet * lp;
                let wet = g.tone_state;

                frame.write(c, dry + (wet * out_lin - dry) * v.mix);
            }
        });
        let db = |p: f32| 20.0 * f64::from(p.max(1e-6)).log10();
        self.taps.io_in.set(db(peaks.input));
        self.taps.io_out.set(db(peaks.output));
    }

    fn face(params: &SmearParams) -> Option<Face> {
        Some(crate::editor::face(params))
    }
}
