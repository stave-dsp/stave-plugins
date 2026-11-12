//! The ribbon ENGINE — a tape echo built as the machine is built. The
//! engine is its COMPONENTS; `process` below is the whole dance:
//! * `preamp.rs` — `Preamp`: the 12AX7 in front of the record head.
//! * `tape.rs` — `Tape`: the loop in tape coordinates — motor, erase,
//!   record, three playback heads.
//! * `spring.rs` — `Spring`: the two-spring dispersive tank.
//! * `telemetry.rs` — `RibbonTaps`: where the loop is, how fast it runs.
//!
//! Signal: dry → preamp → (+ heads × INTENSITY, through the feedback
//! tone) → record head ⟶ tape ⟶ playback heads → head-gap loss, hiss,
//! dropouts → echo; echo and preamp feed the spring; MIX blends the
//! untouched dry against echo + spring.

use stave::dsp::{Coupler, Xorshift32};
use stave::prelude::*;

use crate::params::RibbonParams;

mod preamp;
mod spring;
mod tape;
mod telemetry;

use self::preamp::Preamp;
use self::spring::Spring;
use self::tape::{Tape, Transport};
pub use self::telemetry::RibbonTaps;

/// The HEADS selector: which of heads 1, 2, 3 each position plays.
const HEAD_SETS: [[bool; 3]; 7] = [
    [true, false, false],
    [false, true, false],
    [false, false, true],
    [true, true, false],
    [false, true, true],
    [true, false, true],
    [true, true, true],
];
/// INTENSITY 1 = this much loop gain (runaway starts near 0.8).
const MAX_LOOP_GAIN: f32 = 1.25;
/// Tone coefficients follow the gliding knobs every this many frames.
const CONTROL_FRAMES: usize = 32;
/// The feedback path's tone: bass cut, and a treble cut that WEAR closes.
const FB_LOW_HZ: f32 = 110.0;
const FB_TOP_HZ: [f32; 2] = [6_500.0, 2_400.0];
/// Head-gap loss on playback: fresh … worn.
const GAP_HZ: [f32; 2] = [16_000.0, 4_200.0];
/// WEAR 1: hiss level, dropouts per second.
const HISS: f32 = 0.006;
const DROPOUT_HZ: f32 = 0.9;
/// The record head saturates gently above this.
const REC_CEILING: f32 = 1.0;
/// The spring's drive: the echoes plus some of the preamp.
const SPRING_DRY: f32 = 0.6;
const SPRING_GAIN: f32 = 1.6;
/// The output stage's soft ceiling on the wet.
const WET_CEILING: f32 = 1.4;
/// The SVFs' quality factors: the feedback tone a touch under
/// Butterworth, the head gap a little lower, the hiss tilt at 1/2.
const FB_Q: f32 = 1.0 / 1.8;
const GAP_Q: f32 = 1.0 / 1.7;
const HISS_Q: f32 = 0.5;

#[inline]
fn soft(x: f32, ceiling: f32) -> f32 {
    let x = (x / ceiling).clamp(-3.0, 3.0);
    ceiling * x * (27.0 + x * x) / (27.0 + 9.0 * x * x)
}

#[inline]
fn lerp_log(range: [f32; 2], t: f32) -> f32 {
    range[0] * (range[1] / range[0]).powf(t)
}

#[derive(Smoothers)]
#[smoothers(params = RibbonParams)]
struct Controls {
    intensity: Smoother,
    drive: Smoother,
    wear: Smoother,
    erase: Smoother,
    spring: Smoother,
    mix: Smoother,
    output: Smoother,
}

pub struct Ribbon {
    params: Arc<RibbonParams>,
    controls: Controls,
    sample_rate: f32,
    preamp: Preamp,
    tape: Tape,
    spring: Spring,
    fb_low: [Svf; 2],
    fb_top: [Svf; 2],
    gap: [Svf; 2],
    hiss_tilt: [Svf; 2],
    dc: [Coupler; 2],
    rng: Xorshift32,
    drop_gain: f32,
    drop_target: f32,
    drop_left: u32,
    /// Last frame's echo (the feedback is one frame late, as the
    /// ribbon is: the heads are downstream of the record head).
    echo: [f32; 2],
    taps: RibbonTaps,
}

impl Plugin for Ribbon {
    const INFO: Info = Info {
        id: "com.ryanbahan.stave.ribbon.vst3",
        name: "Stave Ribbon",
        vendor: "Ryan Bahan",
        url: "https://github.com/stave-dsp/stave",
        description: "An impossible tape echo: a real loop, an erase head you can turn down.",
        features: &[Feature::AudioEffect, Feature::Delay, Feature::Reverb, Feature::Stereo],
        ..Info::EFFECT
    };
    type Params = RibbonParams;

    fn new(params: &Arc<RibbonParams>, telemetry: &Telemetry) -> Self {
        Self {
            controls: Controls::new(params),
            params: params.clone(),
            sample_rate: 48_000.0,
            preamp: Preamp::new(),
            tape: Tape::new(),
            spring: Spring::new(),
            fb_low: [Svf::new(FB_LOW_HZ, FB_Q); 2],
            fb_top: [Svf::new(FB_TOP_HZ[0], FB_Q); 2],
            gap: [Svf::new(GAP_HZ[0], GAP_Q); 2],
            hiss_tilt: [Svf::new(1_500.0, HISS_Q); 2],
            dc: [Coupler::new(18.0), Coupler::new(18.0)],
            rng: Xorshift32::new(0x51BB_0042),
            drop_gain: 1.0,
            drop_target: 1.0,
            drop_left: 0,
            echo: [0.0; 2],
            taps: RibbonTaps::new(telemetry),
        }
    }

    fn prepare(&mut self, env: &Environment) {
        let sample_rate = env.sample_rate;
        self.sample_rate = sample_rate;
        self.controls.prepare(env);
        self.preamp.prepare(sample_rate);
        self.tape.prepare(sample_rate, self.params.rate.value());
        self.spring.prepare(sample_rate);
        for f in [&mut self.fb_low, &mut self.fb_top, &mut self.gap, &mut self.hiss_tilt].into_iter().flatten() {
            f.prepare(sample_rate);
        }
        for d in &mut self.dc {
            d.prepare(sample_rate);
        }
        self.rng = Xorshift32::new(0x51BB_0042);
        self.drop_gain = 1.0;
        self.drop_target = 1.0;
        self.drop_left = 0;
        self.echo = [0.0; 2];
    }

    fn process(&mut self, block: &mut Block<'_>) {
        let p = &self.params;
        // 1. Bind: the motor's target, the heads, the block's tone.
        let (rate, brake) = (p.rate.value(), p.brake.value());
        let set = HEAD_SETS[p.heads.value().0 as usize];
        let live = set.iter().filter(|h| **h).count() as f32;
        // Heads sum by equal power on the way out; the feedback takes
        // their AVERAGE, so INTENSITY is the same loop gain whichever
        // heads are live (runaway starts at the same place).
        let head_gain = 1.0 / live.sqrt();
        let fb_norm = 1.0 / (live * head_gain);
        let nyq = 0.45 * self.sample_rate;
        self.preamp.set_drive(p.drive.value());

        // 2. The audio block.
        let ControlsRamps {
            intensity: mut int_ramp,
            drive: mut drive_ramp,
            wear: mut wear_ramp,
            erase: mut erase_ramp,
            spring: mut spring_ramp,
            mix: mut mix_ramp,
            output: mut out_ramp,
        } = self.controls.ramp(block, p);
        let sample_rate = self.sample_rate;
        let (preamp, tape, spring) = (&mut self.preamp, &mut self.tape, &mut self.spring);
        let (fb_low, fb_top, gap, hiss_tilt, dc) =
            (&mut self.fb_low, &mut self.fb_top, &mut self.gap, &mut self.hiss_tilt, &mut self.dc);
        let (rng, drop_gain, drop_target, drop_left) =
            (&mut self.rng, &mut self.drop_gain, &mut self.drop_target, &mut self.drop_left);
        let echo = &mut self.echo;
        let mut frame = 0usize;

        let lanes = block.lanes::<2>();
        let active = lanes.active();
        lanes.run(|fr| {
            let _ = drive_ramp.next();
            let (intensity, wear, erase) = (int_ramp.next(), wear_ramp.next(), erase_ramp.next());
            // WEAR's tone follows the GLIDING knob, not the raw value.
            if frame.is_multiple_of(CONTROL_FRAMES) {
                let (top_hz, gap_hz) = (lerp_log(FB_TOP_HZ, wear).min(nyq), lerp_log(GAP_HZ, wear).min(nyq));
                for ch in 0..2 {
                    fb_top[ch].set(top_hz, FB_Q);
                    gap[ch].set(gap_hz, GAP_Q);
                }
            }
            frame += 1;
            let dry = if active >= 2 { [fr.input(0), fr.input(1)] } else { [fr.input(0); 2] };

            // Record: preamp + the heads coming back round.
            let pre = preamp.tick(dry);
            let g = intensity * MAX_LOOP_GAIN;
            let mut rec = [0.0f32; 2];
            for ch in 0..2 {
                let back = if g > 0.0 { fb_top[ch].tick(fb_low[ch].taps(echo[ch] * g * fb_norm).hp) } else { 0.0 };
                rec[ch] = soft(pre[ch] + back, REC_CEILING);
            }

            // The tape.
            let heads = tape.tick(rec, Transport { rate, brake, erase, wear });
            let mut e = [0.0f32; 2];
            for (h, on) in heads.iter().zip(set) {
                if on {
                    e[0] += h[0];
                    e[1] += h[1];
                }
            }

            // Playback: DC off a stopped tape, head-gap loss, and the
            // worn tape's hiss and dropouts.
            let mut out_e = [0.0f32; 2];
            if wear > 0.0 {
                if *drop_left == 0 {
                    if rng.next_unit() < DROPOUT_HZ * wear * wear / sample_rate {
                        *drop_target = 0.15 + 0.6 * rng.next_unit();
                        *drop_left = ((0.03 + 0.2 * rng.next_unit()) * sample_rate) as u32;
                    } else {
                        *drop_target = 1.0;
                    }
                } else {
                    *drop_left -= 1;
                }
            } else {
                *drop_target = 1.0;
            }
            *drop_gain += (*drop_target - *drop_gain) * (300.0 / sample_rate);
            let hiss_amt = HISS * wear * wear.sqrt();
            for ch in 0..2 {
                let x = gap[ch].tick(dc[ch].tick(e[ch] * head_gain));
                let hiss = if hiss_amt > 0.0 { hiss_amt * hiss_tilt[ch].taps(rng.next_bipolar()).hp } else { 0.0 };
                out_e[ch] = x * *drop_gain + hiss;
            }
            *echo = out_e;

            // The spring, fed by the echoes and the preamp.
            let tank = spring.tick(0.5 * (out_e[0] + out_e[1]) + SPRING_DRY * 0.5 * (pre[0] + pre[1]));
            let s = spring_ramp.next() * SPRING_GAIN;
            // The output stage clips softly, as the machine's did: a
            // runaway loop into a ringing tank saturates, never explodes.
            let wet = [soft(out_e[0] + tank[0] * s, WET_CEILING), soft(out_e[1] + tank[1] * s, WET_CEILING)];

            let a = mix_ramp.next() * core::f32::consts::FRAC_PI_2;
            // MIX 0 is bit-exact: cos(0) = 1 exactly, sin(0) = 0 exactly.
            let (dg, wg) = (a.cos(), a.sin());
            let og = db_to_amp(out_ramp.next());
            if active >= 2 {
                fr.write(0, (dry[0] * dg + wet[0] * wg) * og);
                fr.write(1, (dry[1] * dg + wet[1] * wg) * og);
            } else if active == 1 {
                fr.write(0, (dry[0] * dg + 0.5 * (wet[0] + wet[1]) * wg) * og);
            }
        });

        // 3. What happened, reported out.
        self.taps.publish(self.tape.loop_phase(), self.tape.speed());
    }

    /// The loop forgets only if it is wiped and not re-fed; otherwise
    /// it holds its recording indefinitely.
    fn tail(&self) -> Tail {
        let p = &self.params;
        if p.erase.value() < 0.99 || p.intensity.value() > 0.6 || p.wear.value() > 0.0 {
            Tail::Infinite
        } else {
            Tail::Samples((6.0 * self.sample_rate) as u32)
        }
    }

    fn face(params: &RibbonParams) -> Option<Face> {
        Some(crate::editor::face(params))
    }

    fn presets(params: &RibbonParams) -> Vec<Preset> {
        crate::presets::presets(params)
    }
}
