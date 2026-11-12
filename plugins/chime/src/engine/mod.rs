//! The chime ENGINE — sixteen [`ChimeVoice`]s under a
//! quietest-steal [`VoiceRack`] (the Voice form's first shipping
//! consumer). The engine's own job is exactly the
//! BRIDGE the form's layering law prescribes: host note events →
//! `VoiceEvent` (hz resolved here — tuning is engine business, not
//! kernel business), dispatched at their sample offsets so a note
//! landing mid-block sounds mid-block (a law test).
//!
//! THE MOD MATRIX: routes
//! are ordinary PARAMS (7 per slot × 4), evaluated per block per
//! voice and applied as DISPLACE-FROM-BASE: knobs set the base,
//! routes displace, they never fight. Per-voice targets glide over
//! ~5 ms inside the voice (block-rate application without
//! block-rate zipper).
//!
//! SOURCES (all composition over dsp cells, per the kernel rule):
//! * LFO — shaped (dsp basis; S&H/DRIFT are RandomWalk recipes), tempo-
//!   syncable, NOTE-retriggerable (per-voice phase), with an onset
//!   RISE ramp per note (a player's vibrato fades in);
//! * VEL / KEY / ENV / SPREAD — per-voice: the note's velocity and
//!   key, the voice's own envelope, and a stable per-note random
//!   (unison detune generalized to any target);
//! * HEAT — a fatigue integrator of playing intensity (components
//!   warming up: hit it harder and longer, it drifts);
//! * SYMP — the OTHER voices' energy bleeding in (sympathetic
//!   strings; a lone voice hears none).

use stave::dsp::{
    hz_of_key, mod_lfo_hz, rise, standard_sources, sympathy, Adsr, AdsrConfig, Ensemble, ModLfo,
    ModLfoShape, NoteSources, Osc, Steal, Voice, VoiceEvent, VoiceRack, Waveform,
};
use stave::params::Watch;
use stave::prelude::*;

use crate::params::{ChimeParams, LfoTrig, Shape};

const MAX_VOICES: usize = 16;

/// Voice-sum headroom: −12 dB per voice, so a four-voice fortissimo
/// chord sums to unity and a full 16-voice storm stays inside +12
/// dBFS BEFORE the output trim (the validation suite's param_sweep
/// caught the unscaled sum at +37 dBFS with the trim maxed).
const VOICE_GAIN: f32 = 0.25;

mod matrix;
use matrix::Routes;

/// One note: an oscillator through its envelope, velocity-scaled,
/// displaced by the matrix.
pub struct ChimeVoice {
    osc: Osc,
    env: Adsr,
    /// What this note offers the matrix (velocity, key, spread, age,
    /// its LFO phase).
    sources: NoteSources<1>,
    hz: f32,
    /// Route displacement TARGETS (block rate) and their ~5 ms
    /// glides (frame rate) — displace-from-base without zipper.
    pitch_st_target: f32,
    level_db_target: f32,
    pitch_st: f32,
    level_db: f32,
    glide: f32,
    /// Base envelope knobs (the mod scales DECAY from these).
    base_env: AdsrConfig,
    decay_scale: f32,
    /// Per-slot lag states (the shaping chain's memory).
    lags: [f32; 4],
}

impl ChimeVoice {
    fn new() -> Self {
        Self {
            osc: Osc::new(Waveform::Sine),
            env: Adsr::new(),
            sources: NoteSources::new(0x9E37_79B9),
            hz: 0.0,
            pitch_st_target: 0.0,
            level_db_target: 0.0,
            pitch_st: 0.0,
            level_db: 0.0,
            glide: 1.0,
            base_env: AdsrConfig {
                attack_ms: 5.0,
                decay_ms: 800.0,
                sustain: 0.0,
                release_ms: 1_200.0,
            },
            decay_scale: 1.0,
            lags: [0.0; 4],
        }
    }

    fn with_seed(seed: u32) -> Self {
        let mut v = Self::new();
        v.sources = NoteSources::new(seed);
        v
    }

    /// The voice's own envelope level (the ENV source).
    fn env_level(&self) -> f32 {
        self.env.level()
    }

    /// Block-rate voicing: the envelope knobs and the shape reach
    /// every voice through the rack's `voices_mut`.
    /// The envelope with the matrix's decay displacement applied.
    fn scaled_env(&self) -> AdsrConfig {
        AdsrConfig {
            decay_ms: self.base_env.decay_ms * self.decay_scale,
            ..self.base_env
        }
    }

    fn set(&mut self, env: AdsrConfig, waveform: Waveform) {
        self.base_env = env;
        self.env.set(&self.scaled_env());
        self.osc.set_waveform(waveform);
    }

    /// Evaluate the routes for THIS voice (per-voice sources read
    /// the voice's own state) and set the displacement targets.
    fn apply_routes(&mut self, routes: &Routes, sv: &matrix::SrcValues) {
        let mut lags = self.lags;
        let (pitch, level_db, decay_oct) = matrix::evaluate(routes, sv, &mut lags);
        self.lags = lags;
        self.pitch_st_target = pitch;
        self.level_db_target = level_db;
        let scale = decay_oct.exp2();
        if (scale - self.decay_scale).abs() > 1e-3 {
            self.decay_scale = scale;
            self.env.set(&self.scaled_env());
        }
    }
}

impl Voice for ChimeVoice {
    fn prepare(&mut self, sample_rate: f32) {
        self.osc.prepare(sample_rate);
        self.env.prepare(sample_rate);
        // ~5 ms displacement glide (block-rate targets, no zipper).
        self.glide = 1.0 - (-1.0 / (0.005 * sample_rate)).exp();
        self.reset();
    }
    fn reset(&mut self) {
        self.env.reset();
        self.osc.reset();
        self.sources.velocity = 0.0;
        self.pitch_st = 0.0;
        self.level_db = 0.0;
        self.pitch_st_target = 0.0;
        self.level_db_target = 0.0;
    }
    fn note_on(&mut self, hz: f32, velocity: f32) {
        self.hz = hz;
        self.osc.set_hz(hz);
        self.osc.reset(); // same attack every time: reproducible by construction
        self.env.gate_on();
        self.sources.note_on(hz, velocity);
    }
    fn note_off(&mut self) {
        self.env.gate_off();
    }
    fn choke(&mut self) {
        self.env.choke();
    }
    fn active(&self) -> bool {
        self.env.active()
    }
    fn level(&self) -> f32 {
        self.env.level() * self.sources.velocity
    }
    fn tick(&mut self) -> (f32, f32) {
        self.sources.tick();
        // Displacements glide toward their block-rate targets; the
        // glide is the anti-zipper, not the modulation (routes are
        // the modulation).
        self.pitch_st += self.glide * (self.pitch_st_target - self.pitch_st);
        self.level_db += self.glide * (self.level_db_target - self.level_db);
        if self.pitch_st.abs() > 1e-4 {
            self.osc.set_hz(self.hz * (self.pitch_st / 12.0).exp2());
        }
        let g = if self.level_db.abs() > 1e-4 {
            db_to_amp(self.level_db)
        } else {
            1.0
        };
        let y = self.osc.tick(0.0) * self.env.tick(0.0) * self.sources.velocity * g;
        (y, y)
    }
}

/// The rings' feed: each route slot's live SOURCE value (±1).
#[derive(Taps)]
pub struct ChimeTaps {
    pub mod_1: Tap,
    pub mod_2: Tap,
    pub mod_3: Tap,
    pub mod_4: Tap,
}

impl ChimeTaps {
    /// The four slots' taps, in slot order.
    pub const MODS: [TapId; 4] = [Self::MOD_1, Self::MOD_2, Self::MOD_3, Self::MOD_4];

    fn slots(&self) -> [&Tap; 4] {
        [&self.mod_1, &self.mod_2, &self.mod_3, &self.mod_4]
    }
}

pub struct Chime {
    params: Arc<ChimeParams>,
    output: Smoother,
    sample_rate: f32,
    rack: VoiceRack<ChimeVoice, MAX_VOICES>,
    /// Envelope + shape re-voice only when a knob moved. Reset in
    /// prepare — the ACTIVATION LAW (a re-prepare wipes what the
    /// params can't see).
    voicing: Watch<5>,
    /// The matrix clock (global source; per-voice sources live in
    /// the notes themselves).
    lfo: ModLfo,
    /// HEAT and the last note played, for the rings.
    ensemble: Ensemble,
    /// The rings' feed: per-slot live SOURCE values, as
    /// DECLARED taps. Per-voice sources publish the most
    /// recent note's value — the ring dances to what you just
    /// played, the Vital feel.
    taps: ChimeTaps,
}

impl Plugin for Chime {
    const INFO: Info = Info {
        id: "com.ryanbahan.stave.chime.vst3",
        name: "Stave Chime",
        vendor: "Ryan Bahan",
        url: "https://github.com/stave-dsp/stave",
        description: "A polyphonic sine/saw synth.",
        features: &[Feature::Instrument, Feature::Synthesizer, Feature::Stereo],
        ..Info::INSTRUMENT
    };
    /// An instrument: no audio in, stereo out — stated.
    const LAYOUTS: &'static [Layout] = &[Layout::INSTRUMENT];
    type Params = ChimeParams;

    fn new(params: &Arc<ChimeParams>, telemetry: &Telemetry) -> Self {
        Self {
            output: Smoother::new(&params.output),
            params: params.clone(),
            sample_rate: 48_000.0,
            rack: VoiceRack::new(Steal::Quietest, |i| {
                ChimeVoice::with_seed(0x9E37_79B9 ^ (i as u32).wrapping_mul(0x9E37_79B1))
            }),
            voicing: Watch::new(),
            lfo: ModLfo::new(833.0, 0xC0FF_EE01),
            ensemble: Ensemble::default(),
            taps: ChimeTaps::new(telemetry),
        }
    }

    fn prepare(&mut self, env: &Environment) {
        self.output.prepare(env);
        self.sample_rate = env.sample_rate;
        self.rack.prepare(env.sample_rate);
        self.lfo.prepare(env.sample_rate);
        self.ensemble.reset();
        self.voicing.reset();
    }

    fn reset(&mut self) {
        self.rack.reset();
    }

    fn process(&mut self, block: &mut Block<'_>) {
        let p = &*self.params;
        // 1. Param changes become structure.
        if self
            .voicing
            .changed([&p.attack, &p.decay, &p.sustain, &p.release, &p.shape])
        {
            let env = AdsrConfig {
                attack_ms: p.attack.value(),
                decay_ms: p.decay.value(),
                sustain: p.sustain.value(),
                release_ms: p.release.value(),
            };
            let waveform = if p.shape.value() == Shape(1) {
                Waveform::Saw
            } else {
                Waveform::Sine
            };
            for v in self.rack.voices_mut() {
                v.set(env, waveform);
            }
        }

        // 1b. The matrix: snapshot the route params (7 per slot),
        //     advance the global clocks, then evaluate per voice.
        let block_s = block.frames() as f32 / self.sample_rate;
        let tempo = block.transport().and_then(|t| t.tempo_bpm).unwrap_or(120.0) as f32;
        let hz = mod_lfo_hz(p.lfo_rate.value(), p.lfo_sync.value().to_index() as usize, tempo);
        self.lfo
            .set(hz, ModLfoShape::from_index(p.lfo_shape.value().to_index() as usize));
        let global_lfo = self.lfo.tick_frames(block.frames());
        self.ensemble.leak(block_s);
        let routes = Routes {
            slots: p.routes.each_ref().map(|r| r.slot(block_s)),
        };
        if routes.any() {
            // SYMP needs the ensemble's energy before any voice moves.
            let mut total_level = 0.0f32;
            for v in self.rack.voices_mut() {
                total_level += v.level();
            }
            let rise_ms = p.lfo_rise.value();
            let note_trig = p.lfo_trig.value() == LfoTrig(1);
            let ensemble = &self.ensemble;
            let sample_rate = self.sample_rate;
            let lfo = &self.lfo;
            for v in self.rack.voices_mut() {
                // The voice's own LFO value: NOTE trig reads its own
                // phase, and the RISE ramp fades it in from note-on.
                let raw_lfo = if note_trig {
                    lfo.at_phase(v.sources.lfo_phases[0])
                } else {
                    global_lfo
                };
                v.sources.lfo_phases[0] = lfo.advance_phase(v.sources.lfo_phases[0], block_s);
                let ramp = rise(v.sources.age, rise_ms, sample_rate);
                let sv: matrix::SrcValues = standard_sources(
                    raw_lfo * ramp,
                    &v.sources,
                    v.env_level(),
                    ensemble,
                    sympathy(total_level, v.level()),
                );
                v.apply_routes(&routes, &sv);
            }
        }
        // The rings' feed: each slot's live SOURCE value (±1).
        // Per-voice sources publish the most recent note's or an
        // ensemble view — the ring is an indicator, not a meter.
        let mut total_level = 0.0f32;
        for v in self.rack.voices_mut() {
            total_level += v.level();
        }
        let shown = self.ensemble.sources(global_lfo, total_level);
        for (slot, tap) in routes.slots.iter().zip(self.taps.slots()) {
            tap.set(f64::from(slot.source(&shown).unwrap_or(0.0)));
        }

        // 2. The audio block: the BRIDGE (host events → VoiceEvent,
        //    at their frame offsets) around the rack's tick.
        let mut output = self.output.ramp(block, &p.output);
        let rack = &mut self.rack;
        let mut notes = EventCursor::new(block.events());
        let ensemble = &mut self.ensemble;
        block.lanes::<2>().run(|fr| {
            for event in notes.due(fr.index()) {
                let EventKind::Note(note) = event.kind;
                let voice = note.voice_event(hz_of_key);
                if let VoiceEvent::On { hz, velocity, .. } = voice {
                    ensemble.note_on(hz, velocity);
                }
                rack.dispatch(voice);
            }
            let g = db_to_amp(output.next()) * VOICE_GAIN;
            let (l, r) = rack.tick();
            fr.write(0, l * g);
            fr.write(1, r * g);
        });
    }

    /// Honest from the knobs: the longest a released note can ring
    /// (release parks at −80 dB ≈ 9.3 τ).
    fn tail(&self) -> Tail {
        let r_s = self.params.release.value() * 0.001 * 9.3;
        Tail::Samples((r_s * self.sample_rate) as u32)
    }

    fn face(params: &ChimeParams) -> Option<Face> {
        Some(crate::editor::face(params))
    }
}
