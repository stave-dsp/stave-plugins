//! The eddy ENGINE — a feedback delay whose read head breathes, with
//! four feedback LAWS over one M/S delay substrate (the editor's
//! tabs). The engine is its COMPONENTS — read the fields, know the
//! design; `process` below is the whole per-block dance through them:
//! * `echo.rs` — `Echo`: the shared M/S delay lanes + the classic law.
//! * `motion.rs` — `Motion`: the LFO + the recipe that shapes it.
//! * `circuit.rs` — CIRCUIT: a physics cell at the write head.
//! * `motor.rs` — MOTOR: the wet tail loads the transport; time sags.
//! * `shift.rs` — SHIFT: every repeat transposed by the dialed interval.
//! * `orbit.rs` — ORBIT: each repeat rotates through the image.
//! * `diffusion.rs` — the second tabbed stage, in-loop on the read
//!   side: PLATE / PRISM / DUST / MESH — crank DIFFUSE and the
//!   delay grows into one integrated reverberant system.
//! * `telemetry.rs` — `EddyTaps`: wave cycle, cursor, tail energy.

use stave::prelude::*;

use crate::params::EddyParams;

mod circuit;
mod diffusion;
mod echo;
mod motion;
mod motor;
mod orbit;
mod shift;
mod telemetry;

use circuit::Circuit;
use diffusion::Diffusion;
use echo::Echo;
use motion::{Motion, DEPTH_MS};
use motor::Motor;
use shift::Shift;
pub use telemetry::EddyTaps;

/// Longest dialable delay; the line's capacity adds headroom for the
/// glide, the LFO throw, and MOTOR's full sag (SHIFT's transposer
/// owns its own window lines).
const MAX_TIME_MS: f32 = 2_000.0;

/// Two independent levels, one spelling everywhere. At `dry_g = 1`,
/// `wet_g = 0` this returns `dry` EXACTLY (the bit-dry law); at
/// `dry_g = 0` the dry contributes nothing (the no-dry law).
#[inline]
fn blend(dry: f32, wet: f32, dry_g: f32, wet_g: f32) -> f32 {
    dry * dry_g + wet * wet_g
}

/// The per-frame inputs every algorithm law shares: the stereo frame,
/// the read head (base + wobble + spread), the regen gain, and the
/// diffusion sends. Algorithm-specific knobs stay positional — this
/// carries only what is common to all four laws.
#[derive(Copy, Clone)]
pub(crate) struct FrameCtx {
    pub l: f32,
    pub r: f32,
    /// Read-head center in samples (already time-knob glided).
    pub base: f32,
    /// The motion recipe's throw this frame, in samples.
    pub wobble: f32,
    /// Where the wobble lives: 0 = mid, 1 = side.
    pub spread: f32,
    /// Regeneration gain (capped below unity by declaration).
    pub fb: f32,
    /// Field send for the EAR (what you hear).
    pub diffuse: f32,
    /// Field send for the LOOP (what regenerates).
    pub bleed: f32,
}

/// One `Smoother` per gliding knob, each riding its declared glide.
#[derive(Smoothers)]
#[smoothers(params = EddyParams)]
struct Glides {
    time: Smoother,
    feedback: Smoother,
    dry: Smoother,
    wet: Smoother,
    output: Smoother,
    rate: Smoother,
    depth: Smoother,
    spread: Smoother,
    heat: Smoother,
    sag: Smoother,
    shift: Smoother,
    grain: Smoother,
    splice: Smoother,
    orbit: Smoother,
    diffuse: Smoother,
    bleed: Smoother,
}

pub struct Eddy {
    params: Arc<EddyParams>,
    glides: Glides,
    sample_rate: f32,
    /// The delay lanes (mid/side) and their regeneration.
    echo: Echo,
    /// The read head's breath: LFO × recipe.
    motion: Motion,
    /// CIRCUIT's cells, MOTOR's transport, SHIFT's transposer —
    /// each algorithm keeps its own state; only the active one ticks.
    circuit: Circuit,
    motor: Motor,
    shift: Shift,
    /// The in-loop diffusion stage (the second tab bar).
    diffusion: Diffusion,
    /// Everything reported out: the cursor and tail taps, and the
    /// telemetry the wave cycle is published on.
    taps: EddyTaps,
    telemetry: Arc<Telemetry>,
    /// What the delay loop costs a block (`loop_ns_per_sample`,
    /// `loop_load`): echo, read-head motion, the active algorithm and
    /// the in-loop diffusion all run interleaved in one per-sample
    /// frame law, so the loop is the one stage the perf check can name.
    cost: CostTaps,
}

impl Plugin for Eddy {
    const INFO: Info = Info {
        id: "com.ryanbahan.stave.eddy.vst3",
        name: "Stave Eddy",
        vendor: "Ryan Bahan",
        url: "https://github.com/stave-dsp/stave",
        description: "A feedback delay whose read head breathes.",
        features: &[Feature::AudioEffect, Feature::Delay, Feature::Stereo],
        ..Info::EFFECT
    };
    /// Two delay lanes, one algorithm and the diffusion field a frame:
    /// at the worst knobs ~309 ns a sample a channel on the x86_64
    /// build host; twice that leaves a slower machine room.
    const CPU_BUDGET: Budget = Budget::per_channel(620.0);
    type Params = EddyParams;

    fn new(params: &Arc<EddyParams>, telemetry: &Telemetry) -> Self {
        Self {
            glides: Glides::new(params),
            params: params.clone(),
            sample_rate: 48_000.0,
            echo: Echo::new(),
            motion: Motion::new(),
            circuit: Circuit::new(),
            motor: Motor::new(),
            shift: Shift::new(),
            diffusion: Diffusion::new(),
            taps: EddyTaps::new(telemetry),
            telemetry: telemetry.shared(),
            cost: CostTaps::new(telemetry, cost_tap_ids!("loop")),
        }
    }

    fn prepare(&mut self, env: &Environment) {
        self.glides.prepare(env);
        self.sample_rate = env.sample_rate;
        let reach_ms = MAX_TIME_MS * (1.0 + motor::SAG_MAX) + DEPTH_MS;
        let cap = (reach_ms * 0.001 * env.sample_rate) as usize + 8;
        self.echo.prepare(cap);
        self.motion.prepare(env.sample_rate, self.params.rate.value());
        self.circuit.prepare(env.sample_rate);
        self.motor.prepare(env.sample_rate);
        self.shift.prepare(env.sample_rate);
        self.diffusion.prepare(env.sample_rate);
        self.cost.prepare(env.sample_rate);
    }

    /// The per-block orchestration — three named phases:
    fn process(&mut self, block: &mut Block<'_>) {
        let p = &*self.params;
        // 1. Param changes become structure: the motion recipe
        //    re-resolves only when its knobs moved (republishing the
        //    wave view's picture), circuit voices reset on switch,
        //    and the block-rate machine constants settle.
        if self.motion.apply_changes(p) {
            let (motion, sample_rate) = (&self.motion, self.sample_rate);
            telemetry::publish_cycle(&self.telemetry, |ys| motion.render_cycle(ys, sample_rate));
        }
        self.circuit.apply_changes(p);
        self.diffusion.apply_changes(p);
        self.motor.begin_block(p.inertia.value());
        let algo = p.algo.value().0;

        // 2. The audio block: every knob rides its DECLARED glide;
        //    the lanes scaffolding owns the channel plumbing; the
        //    active algorithm owns the frame law.
        let GlidesRamps {
            mut time,
            mut feedback,
            mut dry,
            mut wet,
            mut output,
            mut rate,
            mut depth,
            mut spread,
            mut heat,
            mut sag,
            shift: mut semitones,
            mut grain,
            mut splice,
            mut orbit,
            mut diffuse,
            mut bleed,
        } = self.glides.ramp(block, p);
        let sample_rate = self.sample_rate;
        let (echo, motion) = (&mut self.echo, &mut self.motion);
        let (circuit, motor, shift) = (&mut self.circuit, &mut self.motor, &mut self.shift);
        let diffusion = &mut self.diffusion;
        let mut wet_peak = 0.0f32;

        let frames = block.frames();
        let lanes = block.lanes::<2>();
        let active = lanes.active();
        self.cost.measure(frames, || lanes.run(|fr| {
            // Ramps honor the declared ranges — no read-site
            // clamps to drift out of sync with the table.
            let base = time.next() * 0.001 * sample_rate;
            let fb = feedback.next();
            let dry_g = dry.next();
            let wet_g = wet.next();
            let gain = db_to_amp(output.next());
            let rate = rate.next();
            let depth = depth.next();
            let spread = spread.next();
            let wobble = motion.tick(rate, depth, sample_rate);
            let diffuse = diffuse.next();
            let bleed = bleed.next();
            if active >= 2 {
                let (l, r) = (fr.input(0), fr.input(1));
                let cx = FrameCtx {
                    l,
                    r,
                    base,
                    wobble,
                    spread,
                    fb,
                    diffuse,
                    bleed,
                };
                let (wet_l, wet_r) = match algo {
                    0 => {
                        let heat = heat.next();
                        let wander = circuit.flutter_tick(heat);
                        let (wl, wr) = echo.stereo_frame_shaped(
                            l,
                            r,
                            base + wander,
                            wobble,
                            spread,
                            fb,
                            |lane, x| circuit.head(lane, x, heat),
                            |m, s| diffusion.couple(m, s, diffuse, bleed),
                        );
                        // The crushed repeats' loudness, paid back
                        // outside the loop (see circuit.rs).
                        let loud = circuit.out_gain(heat);
                        (wl * loud, wr * loud)
                    }
                    1 => motor.frame(echo, diffusion, cx, sag.next()),
                    2 => shift.frame(echo, diffusion, cx, semitones.next(), grain.next(), splice.next()),
                    _ => orbit::frame(echo, diffusion, cx, orbit.next()),
                };
                wet_peak = wet_peak.max(wet_l.abs()).max(wet_r.abs());
                fr.write(0, blend(l, wet_l, dry_g, wet_g) * gain);
                fr.write(1, blend(r, wet_r, dry_g, wet_g) * gain);
            } else if active == 1 {
                // Defensive mono fallback (hosts that ignore the
                // declared stereo layout): the classic law — the
                // algorithms are image machines and need two ears.
                let x = fr.input(0);
                let wet = echo.mono_frame(x, base, wobble, fb);
                wet_peak = wet_peak.max(wet.abs());
                fr.write(0, blend(x, wet, dry_g, wet_g) * gain);
            }
        }));

        // 3. What happened, reported out.
        self.taps.lfo_phase.set(f64::from(self.motion.phase01()));
        self.taps.publish_tail(wet_peak);
    }

    /// The tail is the echo train: time × however many repeats the
    /// current feedback keeps above −60 dB. Honest, from the knobs.
    fn tail(&self) -> Tail {
        let time_s = (self.params.time.value() * 0.001 * self.sample_rate) as u32;
        let fb = self.params.feedback.value();
        if fb <= 0.001 {
            return Tail::Samples(time_s);
        }
        // −60 dB: fb^n < 0.001 → n = ln(0.001)/ln(fb).
        let repeats = (0.001f32.ln() / fb.ln()).ceil().min(400.0) as u32;
        Tail::Samples(time_s.saturating_mul(repeats.max(1)))
    }

    fn face(params: &EddyParams) -> Option<Face> {
        Some(crate::editor::face(params))
    }

    fn presets(params: &EddyParams) -> Vec<Preset> {
        crate::presets::presets(params)
    }
}
