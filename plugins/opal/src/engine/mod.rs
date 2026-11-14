//! The opal ENGINE — sixteen [`OpalVoice`]s under a quietest-steal
//! [`VoiceRack`]; each voice = an [`FmCore`] (four operators, wiring
//! as data) beside FOUR per-voice [`Adsr`] instances — the tier-2
//! shape the mini-FM conformance voice proved before this project
//! existed. The spine is the same three phases as
//! every stave engine: apply changed knobs to the voices, bridge
//! host events to the rack at their sample offsets, tick.

use stave::dsp::{
    Source,
    hz_of_key, mod_lfo_rate, rise, standard_sources, sympathy, tau_coeff, Adsr, AdsrConfig,
    Ensemble, ModLfo, ModLfoShape, NoteSources, Steal, Voice, VoiceEvent, VoiceRack,
};
use stave::prelude::*;

use crate::params::{LfoTrig, OpalParams};

mod matrix;
pub(crate) mod operators;
use matrix::{with_lfo2, Displacements, Routes};
use operators::{FmCore, OpSettings};

const MAX_VOICES: usize = 16;

/// −12 dB voice-sum headroom (the chime lesson: an unscaled pool
/// reads as a param_sweep offender the day someone maxes the trim).
const VOICE_GAIN: f32 = 0.25;

/// One note: the FM core + its four envelope instances.
pub struct OpalVoice {
    core: FmCore,
    envs: [Adsr; 4],
    /// The knobs as last applied, and the voice's settings gliding to
    /// them: levels, feedback and vel mod move a sounding note, where a
    /// step clicks.
    knobs: OpSettings,
    set: OpSettings,
    /// False until the first voicing after prepare, which snaps.
    voiced: bool,
    /// Envelope knob snapshot (a, d, s, r per op) — reapplied to the
    /// per-voice instances whenever the table moves.
    inc: [f64; 4],
    hz: f32,
    /// What this note offers the matrix; its two LFO phases are LFO1
    /// and LFO2 under NOTE trig.
    sources: NoteSources<2>,
    /// Matrix displacement targets (block rate) and their ~5 ms
    /// glides — displace-from-base without zipper (the chime shape).
    disp_target: Displacements,
    disp: Displacements,
    glide: f32,
    sample_rate: f32,
    lags: [f32; 4],
}

impl OpalVoice {
    fn new() -> Self {
        Self {
            core: FmCore::new(),
            envs: core::array::from_fn(|_| Adsr::new()),
            knobs: OpSettings::new(),
            set: OpSettings::new(),
            voiced: false,
            inc: [0.0; 4],
            hz: 0.0,
            sources: NoteSources::new(0x9E37_79B9),
            disp_target: Displacements::default(),
            disp: Displacements::default(),
            glide: 1.0,
            sample_rate: 48_000.0,
            lags: [0.0; 4],
        }
    }

    fn with_seed(seed: u32) -> Self {
        let mut v = Self::new();
        v.sources = NoteSources::new(seed);
        v
    }

    /// The ENV source: the loudest carrier envelope (unscaled).
    fn env_level(&self) -> f32 {
        self.carrier_env_stat(|e| e.level(), f32::max)
    }

    /// Block rate: evaluate the routes for THIS voice.
    fn apply_routes(&mut self, routes: &Routes, sv: &matrix::SrcValues) {
        let mut lags = self.lags;
        self.disp_target = matrix::evaluate(routes, sv, &mut lags);
        self.lags = lags;
    }

    /// Block-rate voicing from the table (via the rack's
    /// `voices_mut`). Envelope times retune live; ratios re-derive
    /// the increments so a ratio knob bends a SOUNDING note.
    fn apply(&mut self, set: OpSettings, adsr: [AdsrConfig; 4]) {
        self.knobs = set;
        let gliding = (self.set.levels, self.set.feedback, self.set.vel_mod);
        self.set = set;
        if self.voiced {
            (self.set.levels, self.set.feedback, self.set.vel_mod) = gliding;
        }
        self.voiced = true;
        for (env, config) in self.envs.iter_mut().zip(adsr) {
            env.set(&config);
        }
        self.retune();
    }

    fn retune(&mut self) {
        for i in 0..4 {
            self.inc[i] = f64::from(self.hz * self.set.ratios[i]) / f64::from(self.sample_rate);
        }
    }

    /// A carrier op reaches the ear; a modulator only shapes one.
    fn carrier_env_stat(&self, f: impl Fn(&Adsr) -> f32, fold: impl Fn(f32, f32) -> f32) -> f32 {
        let carriers = operators::WIRINGS[self.set.algo.min(7)].carriers;
        let mut acc = 0.0f32;
        for (i, env) in self.envs.iter().enumerate() {
            if carriers & (1 << i) != 0 {
                acc = fold(acc, f(env));
            }
        }
        acc
    }
}

impl Voice for OpalVoice {
    fn prepare(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.glide = tau_coeff(ms(5.0), sample_rate);
        for env in &mut self.envs {
            env.prepare(sample_rate);
        }
        self.voiced = false;
        self.reset();
    }
    fn reset(&mut self) {
        self.core.reset();
        for e in self.envs.iter_mut() {
            e.reset();
        }
        self.sources.velocity = 0.0;
        self.disp = Displacements::default();
        self.disp_target = Displacements::default();
    }
    fn note_on(&mut self, hz: f32, velocity: f32) {
        self.hz = hz;
        self.sources.note_on(Hz::new(hz), velocity);
        self.retune();
        self.core.reset(); // same attack every time — reproducible
        for e in self.envs.iter_mut() {
            e.gate_on();
        }
    }
    fn note_off(&mut self) {
        for e in self.envs.iter_mut() {
            e.gate_off();
        }
    }
    fn choke(&mut self) {
        for e in self.envs.iter_mut() {
            e.choke();
        }
    }
    /// Alive while any CARRIER still sounds — a ringing modulator
    /// with dead carriers is silence.
    fn active(&self) -> bool {
        self.carrier_env_stat(|e| if e.active() { 1.0 } else { 0.0 }, f32::max) > 0.0
    }
    fn level(&self) -> f32 {
        self.carrier_env_stat(|e| e.level(), f32::max) * self.sources.velocity
    }
    fn tick(&mut self) -> (f32, f32) {
        self.sources.tick();
        // Glide the matrix displacements (anti-zipper), then apply
        // displace-from-base: knobs are the base, routes displace.
        let g = self.glide;
        for i in 0..4 {
            self.set.levels[i] += g * (self.knobs.levels[i] - self.set.levels[i]);
        }
        self.set.feedback += g * (self.knobs.feedback - self.set.feedback);
        self.set.vel_mod += g * (self.knobs.vel_mod - self.set.vel_mod);
        self.disp.pitch_st += g * (self.disp_target.pitch_st - self.disp.pitch_st);
        self.disp.feedback += g * (self.disp_target.feedback - self.disp.feedback);
        self.disp.level1 += g * (self.disp_target.level1 - self.disp.level1);
        self.disp.ratio2_st += g * (self.disp_target.ratio2_st - self.disp.ratio2_st);
        for i in 0..3 {
            self.disp.levels[i] += g * (self.disp_target.levels[i] - self.disp.levels[i]);
        }
        let mut set = self.set;
        set.levels[0] = (set.levels[0] + self.disp.level1).clamp(0.0, 1.0);
        for i in 0..3 {
            set.levels[i + 1] = (set.levels[i + 1] + self.disp.levels[i]).clamp(0.0, 1.0);
        }
        set.feedback = (set.feedback + self.disp.feedback).clamp(0.0, 1.0);
        let mut inc = if self.disp.pitch_st.abs() > 1e-4 {
            let m = f64::from((self.disp.pitch_st / 12.0).exp2());
            [
                self.inc[0] * m,
                self.inc[1] * m,
                self.inc[2] * m,
                self.inc[3] * m,
            ]
        } else {
            self.inc
        };
        // RATIO 2 bends op 2 alone — inharmonicity, the FM chorus.
        if self.disp.ratio2_st.abs() > 1e-4 {
            inc[1] *= f64::from((self.disp.ratio2_st / 12.0).exp2());
        }
        // Velocity → loudness on carriers, → brightness on
        // modulators (depth = lerp(1, vel, vel_mod)).
        let vel_index = 1.0 + self.set.vel_mod * (self.sources.velocity - 1.0);
        let y = self
            .core
            .tick(&set, &mut self.envs, inc, self.sources.velocity, vel_index);
        (y, y)
    }
}

/// What the face shows live: each route slot's source value.
#[derive(Taps)]
pub struct OpalTaps {
    pub mod_1: Tap,
    pub mod_2: Tap,
    pub mod_3: Tap,
    pub mod_4: Tap,
}

impl OpalTaps {
    /// The route slots' taps, in slot order.
    pub const MODS: [TapId; 4] = [Self::MOD_1, Self::MOD_2, Self::MOD_3, Self::MOD_4];

    fn mods(&self) -> [&Tap; 4] {
        [&self.mod_1, &self.mod_2, &self.mod_3, &self.mod_4]
    }
}

pub struct Opal {
    params: Arc<OpalParams>,
    output: Smoother,
    sample_rate: f32,
    rack: VoiceRack<OpalVoice, MAX_VOICES>,
    /// The whole voicing surface (27 knobs). Reset in prepare — the
    /// PREPARE LAW.
    voicing: Watch<27>,
    /// The matrix clock.
    lfo: ModLfo,
    /// The second LFO (the Absynth lesson); its previous block's value
    /// is what self-referencing rate routes read.
    lfo2: ModLfo,
    /// LFO2 RATE is a GLOBAL-scope target: its routes evaluate once
    /// per block with ensemble source values; the lag states for
    /// that pass live here, not in any voice.
    global_lags: [f32; 4],
    ensemble: Ensemble,
    taps: OpalTaps,
}

impl Plugin for Opal {
    const INFO: Info = Info {
        id: "com.ryanbahan.stave.opal.vst3",
        name: "Stave Opal",
        vendor: "Ryan Bahan",
        url: "https://github.com/stave-dsp/stave",
        description: "A 4-operator FM synth: eight algorithms, velocity into brightness.",
        features: &[Feature::Instrument, Feature::Synthesizer, Feature::Stereo],
        ..Info::INSTRUMENT
    };
    /// An instrument: no audio in, stereo out.
    const LAYOUTS: &'static [Layout] = &[Layout::INSTRUMENT];
    type Params = OpalParams;

    fn new(params: &Arc<OpalParams>, telemetry: &Telemetry) -> Self {
        Self {
            output: Smoother::new(&params.output),
            params: params.clone(),
            sample_rate: 48_000.0,
            rack: VoiceRack::new(Steal::Quietest, |i| {
                OpalVoice::with_seed(0x0BA1_0BA1 ^ (i as u32).wrapping_mul(0x9E37_79B1))
            }),
            voicing: Watch::new(),
            lfo: ModLfo::new(ms(833.0), 0x0BA1_0001),
            lfo2: ModLfo::new(ms(3_333.0), 0x0BA1_0002),
            global_lags: [0.0; 4],
            ensemble: Ensemble::default(),
            taps: OpalTaps::new(telemetry),
        }
    }

    fn prepare(&mut self, env: &Environment) {
        self.output.prepare(env);
        self.sample_rate = env.sample_rate;
        self.rack.prepare(env.sample_rate);
        self.lfo.prepare(env.sample_rate);
        self.lfo2.prepare(env.sample_rate);
        self.global_lags = [0.0; 4];
        self.ensemble.reset();
        self.voicing.reset();
    }

    fn reset(&mut self) {
        self.rack.reset();
    }

    fn process(&mut self, block: &mut Block<'_>) {
        let p = &*self.params;
        // 1. Param changes become structure.
        #[rustfmt::skip]
        let moved = self.voicing.changed([
            &p.algo, &p.feedback, &p.vel_mod,
            &p.ratio1, &p.level1, &p.attack1, &p.decay1, &p.sustain1, &p.release1,
            &p.ratio2, &p.level2, &p.attack2, &p.decay2, &p.sustain2, &p.release2,
            &p.ratio3, &p.level3, &p.attack3, &p.decay3, &p.sustain3, &p.release3,
            &p.ratio4, &p.level4, &p.attack4, &p.decay4, &p.sustain4, &p.release4,
        ]);
        if moved {
            let set = OpSettings {
                ratios: [
                    p.ratio1.value(),
                    p.ratio2.value(),
                    p.ratio3.value(),
                    p.ratio4.value(),
                ],
                levels: [
                    p.level1.value(),
                    p.level2.value(),
                    p.level3.value(),
                    p.level4.value(),
                ],
                algo: p.algo.value().to_index() as usize,
                feedback: p.feedback.value(),
                vel_mod: p.vel_mod.value(),
            };
            let adsr = [
                AdsrConfig {
                    attack_ms: p.attack1.value(),
                    decay_ms: p.decay1.value(),
                    sustain: p.sustain1.value(),
                    release_ms: p.release1.value(),
                },
                AdsrConfig {
                    attack_ms: p.attack2.value(),
                    decay_ms: p.decay2.value(),
                    sustain: p.sustain2.value(),
                    release_ms: p.release2.value(),
                },
                AdsrConfig {
                    attack_ms: p.attack3.value(),
                    decay_ms: p.decay3.value(),
                    sustain: p.sustain3.value(),
                    release_ms: p.release3.value(),
                },
                AdsrConfig {
                    attack_ms: p.attack4.value(),
                    decay_ms: p.decay4.value(),
                    sustain: p.sustain4.value(),
                    release_ms: p.release4.value(),
                },
            ];
            for v in self.rack.voices_mut() {
                v.apply(set, adsr);
            }
        }

        // 1b. The matrix (the chime pattern, per-op destinations,
        //     M-mod sources and shaping).
        let frames = block.frames();
        let block_s = frames as f32 / self.sample_rate;
        let tempo = block
            .transport()
            .and_then(|t| t.tempo_bpm)
            .unwrap_or(120.0) as f32;
        let hz = mod_lfo_rate(p.lfo_rate.value_as(), p.lfo_sync.value().to_index() as usize, tempo);
        self.lfo
            .set(hz, ModLfoShape::from_index(p.lfo_shape.value().to_index() as usize));
        let global_lfo = self.lfo.tick_frames(frames);
        self.ensemble.leak(seconds(f64::from(block_s)));
        let routes = Routes {
            slots: p.routes.each_ref().map(|r| r.slot(block_s)),
        };
        let mut total_level = 0.0f32;
        for v in self.rack.voices_mut() {
            total_level += v.level();
        }
        // LFO 2 — its rate is itself a TARGET, evaluated once per
        // block at GLOBAL scope (ensemble source values; a global
        // clock cannot follow sixteen voices; self-reference reads
        // the PREVIOUS block, the one-block lag every feedback
        // observes).
        let hz2_base = mod_lfo_rate(p.lfo2_rate.value_as(), p.lfo2_sync.value().to_index() as usize, tempo).get();
        let global_sv = with_lfo2(
            self.ensemble.sources(global_lfo, total_level),
            self.lfo2.state(),
        );
        let mut glags = self.global_lags;
        let gdisp = matrix::evaluate(&routes, &global_sv, &mut glags);
        self.global_lags = glags;
        let hz2 = (hz2_base * gdisp.lfo2_oct.exp2()).clamp(0.01, 40.0);
        self.lfo2
            .set(Hz::new(hz2), ModLfoShape::from_index(p.lfo2_shape.value().to_index() as usize));
        let global_lfo2 = self.lfo2.tick_frames(frames);
        if routes.any() {
            let rise_ms = p.lfo_rise.value();
            let rise2_ms = p.lfo2_rise.value();
            let note_trig = p.lfo_trig.value() == LfoTrig(1);
            let note_trig2 = p.lfo2_trig.value() == LfoTrig(1);
            let ensemble = &self.ensemble;
            let sample_rate = self.sample_rate;
            let (lfo, lfo2) = (&self.lfo, &self.lfo2);
            for v in self.rack.voices_mut() {
                let raw_lfo = if note_trig {
                    lfo.at_phase(v.sources.lfo_phases[0])
                } else {
                    global_lfo
                };
                let raw_lfo2 = if note_trig2 {
                    lfo2.at_phase(v.sources.lfo_phases[1])
                } else {
                    global_lfo2
                };
                let phases = &mut v.sources.lfo_phases;
                phases[0] = lfo.advance_phase(phases[0], seconds(f64::from(block_s)));
                phases[1] = lfo2.advance_phase(phases[1], seconds(f64::from(block_s)));
                let age = v.sources.age;
                let sv = with_lfo2(
                    standard_sources(
                        raw_lfo * rise(age, ms(f64::from(rise_ms)), sample_rate),
                        &v.sources,
                        v.env_level(),
                        ensemble,
                        sympathy(total_level, v.level()),
                    ),
                    raw_lfo2 * rise(age, ms(f64::from(rise2_ms)), sample_rate),
                );
                v.apply_routes(&routes, &sv);
            }
        }
        let shown = with_lfo2(self.ensemble.sources(global_lfo, total_level), global_lfo2);
        for (tap, slot) in self.taps.mods().into_iter().zip(&routes.slots) {
            tap.set(f64::from(slot.source(&shown).unwrap_or(0.0)));
        }

        // 2. The audio block: the host-event → VoiceEvent bridge at
        //    sample offsets, around the rack's tick.
        let mut output = self.output.ramp(block, &p.output);
        let rack = &mut self.rack;
        let mut notes = EventCursor::new(block.events());
        let ensemble = &mut self.ensemble;
        block.lanes::<2>().run(|fr| {
            for event in notes.due(fr.index()) {
                let EventKind::Note(note) = event.kind;
                let voice = note.voice_event(hz_of_key);
                if let VoiceEvent::On { hz, velocity, .. } = voice {
                    ensemble.note_on(Hz::new(hz), velocity);
                }
                rack.dispatch(voice);
            }
            let g = db_to_amp(output.next()) * VOICE_GAIN;
            let (l, r) = rack.tick();
            fr.write(0, l * g);
            fr.write(1, r * g);
        });
    }

    fn face(params: &OpalParams) -> Option<Face> {
        Some(crate::editor::face(params))
    }

    fn presets(params: &OpalParams) -> Vec<Preset> {
        crate::presets::presets(params)
    }

    /// The longest a released note can ring: the slowest carrier
    /// release, to the −80 dB park (≈ 9.3 τ).
    fn tail(&self) -> Tail {
        let r = self
            .params
            .release1
            .value()
            .max(self.params.release2.value())
            .max(self.params.release3.value())
            .max(self.params.release4.value());
        Tail::Samples((r * 0.001 * 9.3 * self.sample_rate) as u32)
    }
}
