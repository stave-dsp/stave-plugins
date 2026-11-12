//! The saga ENGINE — a two-machine ambient generator modelled on a
//! dual-DSP reverse/reverb pedal (docs/saga.md). The engine is its
//! COMPONENTS; `process` below is the whole per-block dance:
//! * `motion.rs` — `Motion`: the clock — base, DIVE/RISE glide, MOD LFO.
//! * `clock.rs` — `ClockDomain`: host rate ⇄ the 32 kHz core clock.
//! * `programs.rs` — `Programs`: the five programs (two machines in series,
//!   two feedback paths), built from the cells:
//!   `reverse.rs` (ReverseLine), `tank.rs` (Tank), `transpose.rs`
//!   (Transposer), `path.rs` (AnalogPath).
//! * `wet.rs` — `Wet`: TONE, WIDTH, the soft ceiling.
//! * `telemetry.rs` — `SagaTaps`: clock + tail out.

use stave::prelude::*;

use crate::params::SagaParams;

mod clock;
mod motion;
mod path;
mod programs;
mod reverse;
mod tank;
mod telemetry;
mod transpose;
mod wet;

use self::clock::ClockDomain;
use self::programs::{ms_to_ticks, Knobs, Programs, MAX_TIME_MS};
use self::motion::Motion;
use self::wet::Wet;

pub use self::telemetry::SagaTaps;

/// TIME's floor in ms (params.rs) — program IV reads the knob's
/// position between the two.
const MIN_TIME_MS: f32 = 50.0;
/// Beats per SYNC division (index 0 = FREE, unused).
const SYNC_BEATS: [f32; 7] = [0.0, 2.0, 1.0, 0.75, 0.5, 1.0 / 3.0, 0.25];
/// A program change fades the wet out, wipes the core, fades back.
const SWITCH_FADE_MS: f32 = 25.0;

/// One `Smoother` per gliding knob.
struct Controls {
    feedback: Smoother,
    regen: Smoother,
    mod_depth: Smoother,
    mix: Smoother,
    x: Smoother,
    time: Smoother,
    tone: Smoother,
    width: Smoother,
    mod_rate: Smoother,
    output: Smoother,
}

impl Controls {
    fn new(p: &SagaParams) -> Self {
        Self {
            feedback: Smoother::new(&p.feedback),
            regen: Smoother::new(&p.regen),
            mod_depth: Smoother::new(&p.mod_depth),
            mix: Smoother::new(&p.mix),
            x: Smoother::new(&p.x),
            time: Smoother::new(&p.time),
            tone: Smoother::new(&p.tone),
            width: Smoother::new(&p.width),
            mod_rate: Smoother::new(&p.mod_rate),
            output: Smoother::new(&p.output),
        }
    }

    fn prepare(&mut self, env: &Environment) {
        for s in [
            &mut self.feedback,
            &mut self.regen,
            &mut self.mod_depth,
            &mut self.mix,
            &mut self.x,
            &mut self.time,
            &mut self.tone,
            &mut self.width,
            &mut self.mod_rate,
            &mut self.output,
        ] {
            s.prepare(env);
        }
    }
}

/// The plugin: the clock, the programs, the wet stage.
pub struct Saga {
    params: Arc<SagaParams>,
    controls: Controls,
    sample_rate: f32,
    /// The program the core is wired as (lags `mode` across a switch).
    mode: usize,
    /// Wet fade 0..1 around program switches.
    fade: f32,
    motion: Motion,
    clock: ClockDomain,
    programs: Programs,
    wet: Wet,
    taps: SagaTaps,
    last_clock: f32,
}

impl Plugin for Saga {
    const INFO: Info = Info {
        id: "com.ryanbahan.stave.saga.vst3",
        name: "Stave Saga",
        vendor: "Ryan Bahan",
        url: "https://github.com/stave-dsp/stave",
        description: "Reverse delays and reverbs in series, on a clock you can pull.",
        features: &[Feature::AudioEffect, Feature::Reverb, Feature::Delay, Feature::Stereo],
        ..Info::EFFECT
    };
    type Params = SagaParams;

    fn new(params: &Arc<SagaParams>, telemetry: &Telemetry) -> Self {
        Self {
            controls: Controls::new(params),
            params: params.clone(),
            sample_rate: 48_000.0,
            mode: 0,
            fade: 1.0,
            motion: Motion::new(),
            clock: ClockDomain::new(),
            programs: Programs::new(),
            wet: Wet::new(),
            taps: SagaTaps::new(telemetry),
            last_clock: 1.0,
        }
    }

    fn prepare(&mut self, env: &Environment) {
        self.sample_rate = env.sample_rate;
        self.controls.prepare(env);
        self.mode = self.params.mode.value().0 as usize;
        self.fade = 1.0;
        self.last_clock = 1.0;
        self.motion.prepare(env.sample_rate);
        self.clock.prepare(env.sample_rate);
        self.programs.prepare();
        self.wet.prepare(env.sample_rate);
    }

    fn process(&mut self, block: &mut Block<'_>) {
        let p = self.params.clone();
        // 1. Bind: the program, its voicing, the converter filters at
        //    the clock the last block ended on, and where TIME comes
        //    from (the knob, or the host tempo).
        let wanted = p.mode.value().0 as usize;
        self.programs.voice(self.mode, p.x.value());
        self.clock.set_clock(self.last_clock);
        let synced_ms = match (p.sync.value().0 as usize, block.transport().and_then(|t| t.tempo_bpm)) {
            (0, _) | (_, None) => None,
            (div, Some(bpm)) => Some(
                (60_000.0 / bpm as f32 * SYNC_BEATS[div]).clamp(MIN_TIME_MS, MAX_TIME_MS),
            ),
        };
        let shape = p.mod_shape.value().0 as usize;
        let (shift, ramp, held) = (p.shift.value(), p.ramp.value(), p.dive.value());

        // 2. The audio block.
        let c = &mut self.controls;
        let mut time_ramp = c.time.ramp(block, &p.time);
        let mut fb_ramp = c.feedback.ramp(block, &p.feedback);
        let mut regen_ramp = c.regen.ramp(block, &p.regen);
        let mut x_ramp = c.x.ramp(block, &p.x);
        let mut mix_ramp = c.mix.ramp(block, &p.mix);
        let mut tone_ramp = c.tone.ramp(block, &p.tone);
        let mut width_ramp = c.width.ramp(block, &p.width);
        let mut depth_ramp = c.mod_depth.ramp(block, &p.mod_depth);
        let mut rate_ramp = c.mod_rate.ramp(block, &p.mod_rate);
        let mut out_ramp = c.output.ramp(block, &p.output);
        let fade_step = 1000.0 / (SWITCH_FADE_MS * self.sample_rate);
        let (motion, clock, programs, wet) = (&mut self.motion, &mut self.clock, &mut self.programs, &mut self.wet);
        let (mode, fade) = (&mut self.mode, &mut self.fade);
        let mut clk = self.last_clock;
        let mut wet_peak = 0.0f32;

        let lanes = block.lanes::<2>();
        let active = lanes.active();
        lanes.run(|fr| {
            let knob_ms = time_ramp.next();
            let time_ms = synced_ms.unwrap_or(knob_ms);
            let k = Knobs {
                feedback: fb_ramp.next(),
                regen: regen_ramp.next(),
                x: x_ramp.next(),
                len: ms_to_ticks(time_ms),
            };
            let base = if *mode == 3 {
                Motion::stretch((time_ms / MIN_TIME_MS).ln() / (MAX_TIME_MS / MIN_TIME_MS).ln())
            } else {
                1.0
            };
            clk = motion.tick(base, shift, ramp, held, depth_ramp.next(), rate_ramp.next(), shape);

            let dry = if active >= 2 { [fr.input(0), fr.input(1)] } else { [fr.input(0); 2] };
            let w = clock.process(dry, clk, |xi| programs.tick(xi, k));
            let w = wet.process(w, tone_ramp.next(), width_ramp.next());

            // Program switch: fade out, rewire on silence, fade in.
            if wanted != *mode {
                *fade -= fade_step;
                if *fade <= 0.0 {
                    *fade = 0.0;
                    *mode = wanted;
                    programs.reset();
                    programs.voice(wanted, k.x);
                }
            } else if *fade < 1.0 {
                *fade = (*fade + fade_step).min(1.0);
            }

            let m = mix_ramp.next() * std::f32::consts::FRAC_PI_2;
            let (dry_g, wet_g) = (m.cos(), m.sin() * *fade);
            let g = db_to_amp(out_ramp.next());
            wet_peak = wet_peak.max(w[0].abs()).max(w[1].abs());
            if active >= 2 {
                fr.write(0, (dry[0] * dry_g + w[0] * wet_g) * g);
                fr.write(1, (dry[1] * dry_g + w[1] * wet_g) * g);
            } else if active == 1 {
                fr.write(0, (dry[0] * dry_g + 0.5 * (w[0] + w[1]) * wet_g) * g);
            }
        });
        self.last_clock = clk;

        // 3. What happened, reported out.
        self.taps.publish(clk, wet_peak);
    }

    /// The loops are soft-clipped, so near-unity feedback sustains
    /// indefinitely; below that the longest tank (25 s RT60) bounds it.
    fn tail(&self) -> Tail {
        let p = &self.params;
        if p.feedback.value() > 0.9 || p.regen.value() > 0.9 {
            Tail::Infinite
        } else {
            Tail::Samples((30.0 * self.sample_rate) as u32)
        }
    }

    fn face(params: &SagaParams) -> Option<Face> {
        Some(crate::editor::face(params))
    }

    fn presets(params: &SagaParams) -> Vec<Preset> {
        crate::presets::presets(params)
    }
}
