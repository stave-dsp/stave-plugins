//! The heirloom ENGINE — saga's reverse machines played back off a
//! worn record (docs/design.md). The engine is its COMPONENTS;
//! `process` below is the whole per-block dance:
//! * `motion.rs` — `Motion`: the core clock — base, HOLD's sag, MOD LFO.
//! * `clock.rs` — `ClockDomain`: host rate ⇄ the 32 kHz core clock.
//! * `programs.rs` — `Programs`: the reverse machines (saga's
//!   programs), built from `reverse.rs`, `tank.rs`, `transpose.rs`,
//!   `path.rs`.
//! * `wet.rs` — `Wet`: tone, width, the soft ceiling.
//! * `media.rs` — `Media`: WOBBLE + AGE — the record the wet path
//!   (dry + machine) plays off; MIX blends it against the clean dry.
//! * `telemetry.rs` — `HeirloomTaps`: clock, tail and spin out.
//!
//! The face is six knobs; the core's other controls are VOICED from
//! them (`voicing` below) — the pedal decides, the player plays.

use stave::prelude::*;

use crate::params::HeirloomParams;

mod clock;
mod media;
mod motion;
mod path;
mod programs;
mod reverse;
mod tank;
mod telemetry;
mod transpose;
mod wet;

use self::clock::ClockDomain;
use self::media::{Media, Wear};
use self::motion::Motion;
use self::programs::{ms_to_ticks, Knobs, Programs, MAX_TIME_MS};
use self::wet::Wet;

pub use self::telemetry::HeirloomTaps;

/// TIME's floor in ms (params.rs) — BLOOM reads the knob's position
/// between the two.
const MIN_TIME_MS: f32 = 50.0;
/// Beats per SYNC division (index 0 = FREE, unused).
const SYNC_BEATS: [f32; 7] = [0.0, 2.0, 1.0, 0.75, 0.5, 1.0 / 3.0, 0.25];
/// A machine change fades the wet out, wipes the core, fades back.
const SWITCH_FADE_MS: f32 = 25.0;

/// The voicing: what the face's three-way MACHINE toggle wires.
/// (saga program, X — how reversed for REVERSE, decay for the others)
const MACHINES: [(usize, f32); 3] = [(0, 0.85), (1, 0.6), (3, 0.7)];
/// REPEATS drives the first loop; the second rides a little under it.
const REGEN_OF_REPEATS: f32 = 0.9;
/// WOBBLE also detunes the core clock — the tail smears with the record.
const CORE_MOD_OF_WOBBLE: f32 = 0.15;
/// The wet image's spread.
const WIDTH: f32 = 0.7;
/// The dry and the machine meet on the record at −3 dB each, so a
/// full-wet pedal sits near the dry's level.
const WET_SUM: f32 = std::f32::consts::FRAC_1_SQRT_2;
/// HOLD sags the clock an octave over this long (a tape stop's glide).
const HOLD_SHIFT: f32 = 0.5;
const HOLD_RAMP_MS: f32 = 700.0;
/// The platter: 33⅓ rpm, spun up and braked like a real deck.
const PLATTER_REV_HZ: f64 = 33.333 / 60.0;
const PLATTER_START_S: f64 = 0.5;
const PLATTER_BRAKE_S: f64 = 1.4;

/// The wet's tone darkens as the record ages.
#[inline]
fn tone_of_age(age: f32) -> f32 {
    0.85 - 0.3 * age
}

/// One `Smoother` per gliding knob.
struct Controls {
    time: Smoother,
    repeats: Smoother,
    mix: Smoother,
    wobble: Smoother,
    rpm: Smoother,
    age: Smoother,
    output: Smoother,
}

impl Controls {
    fn new(p: &HeirloomParams) -> Self {
        Self {
            time: Smoother::new(&p.time),
            repeats: Smoother::new(&p.repeats),
            mix: Smoother::new(&p.mix),
            wobble: Smoother::new(&p.wobble),
            rpm: Smoother::new(&p.rpm),
            age: Smoother::new(&p.age),
            output: Smoother::new(&p.output),
        }
    }

    fn prepare(&mut self, env: &Environment) {
        for s in [
            &mut self.time,
            &mut self.repeats,
            &mut self.mix,
            &mut self.wobble,
            &mut self.rpm,
            &mut self.age,
            &mut self.output,
        ] {
            s.prepare(env);
        }
    }
}

/// The plugin: saga's clock and programs, the wet stage, the record.
pub struct Heirloom {
    params: Arc<HeirloomParams>,
    controls: Controls,
    sample_rate: f32,
    /// The machine the core is wired as (lags `machine` across a switch).
    machine: usize,
    /// Wet fade 0..1 around machine switches.
    fade: f32,
    motion: Motion,
    clock: ClockDomain,
    programs: Programs,
    wet: Wet,
    media: Media,
    taps: HeirloomTaps,
    last_clock: f32,
    /// The record the face shows: angle (radians) and speed (1 = 33⅓).
    spin: f64,
    spin_speed: f64,
}

impl Plugin for Heirloom {
    const INFO: Info = Info {
        id: "com.ryanbahan.stave.heirloom.vst3",
        name: "Stave Heirloom",
        vendor: "Ryan Bahan",
        url: "https://github.com/stave-dsp/stave",
        description: "Reverse machines played back off a worn record.",
        features: &[Feature::AudioEffect, Feature::Reverb, Feature::Delay, Feature::Stereo],
        ..Info::EFFECT
    };
    type Params = HeirloomParams;

    fn new(params: &Arc<HeirloomParams>, telemetry: &Telemetry) -> Self {
        Self {
            controls: Controls::new(params),
            params: params.clone(),
            sample_rate: 48_000.0,
            machine: 0,
            fade: 1.0,
            motion: Motion::new(),
            clock: ClockDomain::new(),
            programs: Programs::new(),
            wet: Wet::new(),
            media: Media::new(),
            taps: HeirloomTaps::new(telemetry),
            last_clock: 1.0,
            spin: 0.0,
            spin_speed: 0.0,
        }
    }

    fn prepare(&mut self, env: &Environment) {
        self.sample_rate = env.sample_rate;
        self.controls.prepare(env);
        self.machine = self.params.machine.value().0 as usize;
        self.fade = 1.0;
        self.last_clock = 1.0;
        self.motion.prepare(env.sample_rate);
        self.clock.prepare(env.sample_rate);
        self.programs.prepare();
        self.wet.prepare(env.sample_rate);
        self.media.prepare(env.sample_rate);
    }

    fn process(&mut self, block: &mut Block<'_>) {
        let p = self.params.clone();
        // 1. Bind: the machine, its voicing, the converter filters at
        //    the clock the last block ended on, where TIME comes from
        //    (the knob, or the host tempo), and the medium's corners.
        let wanted = p.machine.value().0 as usize;
        let (program, x) = MACHINES[self.machine];
        self.programs.voice(program, x);
        self.clock.set_clock(self.last_clock);
        let synced_ms = match (p.sync.value().0 as usize, block.transport().and_then(|t| t.tempo_bpm)) {
            (0, _) | (_, None) => None,
            (div, Some(bpm)) => Some(
                (60_000.0 / bpm as f32 * SYNC_BEATS[div]).clamp(MIN_TIME_MS, MAX_TIME_MS),
            ),
        };
        let warp = p.warp.value().0 as usize;
        // DRIFT wanders the core clock at random too; the other two
        // shapes are the media stage's own — the core takes a sine.
        let core_shape = if warp == 2 { 2 } else { 0 };
        let held = p.hold.value();
        let wear = Wear { media: p.media.value().0 as usize, warp };
        self.media.begin_block(p.age.value(), wear);

        // 2. The audio block.
        let c = &mut self.controls;
        let mut time_ramp = c.time.ramp(block, &p.time);
        let mut repeats_ramp = c.repeats.ramp(block, &p.repeats);
        let mut mix_ramp = c.mix.ramp(block, &p.mix);
        let mut wobble_ramp = c.wobble.ramp(block, &p.wobble);
        let mut rpm_ramp = c.rpm.ramp(block, &p.rpm);
        let mut age_ramp = c.age.ramp(block, &p.age);
        let mut out_ramp = c.output.ramp(block, &p.output);
        let fade_step = 1000.0 / (SWITCH_FADE_MS * self.sample_rate);
        let (motion, clock, programs, wet, media) =
            (&mut self.motion, &mut self.clock, &mut self.programs, &mut self.wet, &mut self.media);
        let (machine, fade) = (&mut self.machine, &mut self.fade);
        let mut clk = self.last_clock;
        let mut wet_peak = 0.0f32;

        let frames = block.frames();
        let lanes = block.lanes::<2>();
        let active = lanes.active();
        lanes.run(|fr| {
            let knob_ms = time_ramp.next();
            let time_ms = synced_ms.unwrap_or(knob_ms);
            let repeats = repeats_ramp.next();
            let (wobble, rpm, age) = (wobble_ramp.next(), rpm_ramp.next(), age_ramp.next());
            let (program, x) = MACHINES[*machine];
            let k = Knobs {
                feedback: repeats,
                regen: repeats * REGEN_OF_REPEATS,
                x,
                len: ms_to_ticks(time_ms),
            };
            let base = if program == 3 {
                Motion::stretch((time_ms / MIN_TIME_MS).ln() / (MAX_TIME_MS / MIN_TIME_MS).ln())
            } else {
                1.0
            };
            clk = motion.tick(
                base,
                HOLD_SHIFT,
                HOLD_RAMP_MS,
                held,
                wobble * CORE_MOD_OF_WOBBLE,
                rpm,
                core_shape,
            );

            let dry = if active >= 2 { [fr.input(0), fr.input(1)] } else { [fr.input(0); 2] };
            let w = clock.process(dry, clk, |xi| programs.tick(xi, k));
            let w = wet.process(w, tone_of_age(age), WIDTH);

            // Machine switch: fade out, rewire on silence, fade in.
            if wanted != *machine {
                *fade -= fade_step;
                if *fade <= 0.0 {
                    *fade = 0.0;
                    *machine = wanted;
                    programs.reset();
                    programs.voice(MACHINES[wanted].0, MACHINES[wanted].1);
                }
            } else if *fade < 1.0 {
                *fade = (*fade + fade_step).min(1.0);
            }

            // The WET path is the whole pedal: the dry and the reverse
            // machine together, played off the worn record. MIX blends
            // that against the untouched dry — at 50% the wobble beats
            // against the dry (the Warped Vinyl chorus).
            let rec = [
                (dry[0] + w[0] * *fade) * WET_SUM,
                (dry[1] + w[1] * *fade) * WET_SUM,
            ];
            let worn = media.process(rec, wobble, rpm, age, wear);
            wet_peak = wet_peak.max(w[0].abs()).max(w[1].abs());
            let m = mix_ramp.next() * std::f32::consts::FRAC_PI_2;
            let (dry_g, wet_g) = (m.cos(), m.sin());
            // MIX 0 is bit-exact: cos(0) = 1 exactly and a wet gain of
            // exactly 0 contributes exactly 0 — whatever the media does.
            let y = [dry[0] * dry_g + worn[0] * wet_g, dry[1] * dry_g + worn[1] * wet_g];
            let g = db_to_amp(out_ramp.next());
            if active >= 2 {
                fr.write(0, y[0] * g);
                fr.write(1, y[1] * g);
            } else if active == 1 {
                fr.write(0, 0.5 * (y[0] + y[1]) * g);
            }
        });
        self.last_clock = clk;

        // 3. The platter: it turns at the speed the audio PLAYS (the
        //    wobble's lurches, HOLD's sag) and brakes to a stop when
        //    the pedal is bypassed.
        let dt = frames as f64 / f64::from(self.sample_rate);
        let target = if p.bypass.value() { 0.0 } else { f64::from(clk * self.media.take_speed()) };
        let tau = if target < self.spin_speed { PLATTER_BRAKE_S } else { PLATTER_START_S };
        self.spin_speed += (target - self.spin_speed) * (1.0 - (-dt / tau).exp());
        self.spin = (self.spin + self.spin_speed * std::f64::consts::TAU * PLATTER_REV_HZ * dt)
            .rem_euclid(std::f64::consts::TAU);

        // 4. What happened, reported out.
        self.taps.publish(clk, wet_peak, self.spin);
    }

    /// The loops are soft-clipped, so near-unity repeats sustain
    /// indefinitely; AGE's noise floor never stops either.
    fn tail(&self) -> Tail {
        let p = &self.params;
        if p.repeats.value() > 0.9 || p.age.value() > 0.0 {
            Tail::Infinite
        } else {
            Tail::Samples((30.0 * self.sample_rate) as u32)
        }
    }

    fn face(params: &HeirloomParams) -> Option<Face> {
        Some(crate::editor::face(params))
    }

    fn presets(params: &HeirloomParams) -> Vec<Preset> {
        crate::presets::presets(params)
    }
}
