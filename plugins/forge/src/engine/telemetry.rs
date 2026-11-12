//! The observer — everything the plugin reports OUT on its
//! `Telemetry`: declared taps, the capture pairs, the transfer curve,
//! and the per-block publish pass.

use std::time::Instant;

use stave::dsp::{shapes, StageKind};
use stave::prelude::*;
use stave::telemetry::{Scope, CURVE_POINTS, CURVE_X_SPAN};

use crate::engine::voicing::{character_shape, stage_b_knee};
use crate::params::ForgeParams;

#[inline]
fn peak_db(peak: f32) -> f64 {
    20.0 * (peak.max(1e-6) as f64).log10()
}

/// Everything the plugin reports out, as ONE component: declared
/// tap handles, the framework capture scope, and the editor-feed
/// caches. The strips process audio; the observer publishes what
/// happened. Nothing here is DSP state.
pub(super) struct Observer {
    telemetry: Arc<Telemetry>,
    taps: ForgeTaps,
    /// Framework capture scratch + load meter.
    scope: Scope,
    /// Voicing tuple last published (transfer view republish gate).
    last_voicing: (i32, i32, f32, f32),
    /// Screamer input gain (primary is an RC there — no Frame to read).
    sc_gain: f32,
    /// Per-stage block peaks of the mid chain's sense endpoints
    /// (the topology strip's feed).
    tap_peaks: [f32; 6],
}

/// One processed block, as the per-block publish pass reports it.
pub(super) struct BlockReport {
    pub frames: usize,
    pub peak_in: f32,
    pub peak_out: f32,
    pub started: Instant,
}

/// The voicing tuple the static transfer view is drawn from.
pub(super) struct Voicing {
    pub character: usize,
    pub stack: usize,
    pub edge: f32,
    pub bias: f32,
    /// The primary frame's bias.
    pub frame_bias: f32,
    pub drive: f32,
}

impl Observer {
    /// Taps registered and the capture claimed: the scope wants
    /// transfer PAIRS, not output L/R (ch0 = shaper-domain input,
    /// ch1 = chain output).
    pub(super) fn new(telemetry: &Telemetry) -> Self {
        telemetry.claim_capture();
        Self {
            telemetry: telemetry.shared(),
            taps: ForgeTaps::new(telemetry),
            scope: Scope::new(),
            last_voicing: (-1, -1, -1.0, -1.0),
            sc_gain: 1.0,
            tap_peaks: [0.0; 6],
        }
    }
}

impl Observer {
    pub(super) fn prepare(&mut self, max_frames: usize) {
        self.scope.prepare(max_frames);
    }
    /// Start-of-block reset for the per-stage peak accumulators.
    pub(super) fn begin_block(&mut self) {
        self.tap_peaks = [0.0; 6];
    }
    /// Scope reference for this frame: the LIVE frame's (bias, drive)
    /// on the primary stage — or the screamer's input gain when the
    /// primary is an RC (no Frame to read).
    #[inline]
    pub(super) fn scope_reference(&self, strip: &mut super::strip::Strip) -> (f32, f32) {
        let id_a = strip.amp().primary;
        if let Some(StageKind::Curve(cs)) =
            strip.amp_mut().as_chain_mut().stage_mut(id_a)
        {
            let fr = *cs.pending_mut();
            (fr.bias, fr.drive)
        } else {
            (0.0, self.sc_gain)
        }
    }
    /// One frame's capture: the scope's transfer pair plus the
    /// topology strip's sense-endpoint peaks (derived from
    /// `topology()`, never an index literal).
    #[inline]
    pub(super) fn capture(&mut self, frame: usize, x: f32, y: f32, strip: &super::strip::Strip) {
        self.scope.write(frame, x, y);
        for (id, _) in strip.amp().as_chain().topology() {
            self.tap_peaks[id.0] = self.tap_peaks[id.0].max(strip.amp().as_chain().sense(id).abs());
        }
    }
}

impl Observer {
    /// The per-block publish pass: block peaks, the scope's capture
    /// pair, per-stage topology peaks, the load meter.
    pub(super) fn publish(&mut self, p: &ForgeParams, sample_rate: f32, block: BlockReport) {
        let BlockReport {
            frames,
            peak_in: peak_in_block,
            peak_out: peak_out_block,
            started: t_block,
        } = block;
        let taps = &self.taps;
        taps.io_in.set(peak_db(peak_in_block));
        taps.io_out.set(peak_db(peak_out_block));
        // Plugin view: transfer pairs for the phosphor scope…
        self.scope.publish(&self.telemetry, frames);
        // …and per-stage tap peaks for the topology strip, on STABLE
        // wires (values only — the face's pill declarations own the
        // labels). Stage 2's pill hides in SINGLE by not publishing
        // (the strip ages it out).
        let t = self.tap_peaks;
        let single = p.stack.value().0 == 0;
        for (k, tap) in taps.stages().into_iter().enumerate() {
            if k == 2 && single {
                continue;
            }
            tap.set(f64::from(t[k]));
        }
        if frames > 0 {
            let budget = frames as f64 / f64::from(sample_rate);
            let pct = self.scope.load_pct(t_block.elapsed().as_secs_f64(), budget);
            taps.cpu_pct.set(pct);
        }
    }
}

impl Observer {
    /// Static transfer view: the curve stack's skeleton (the voicing
    /// filters and memory bend it dynamically) plus the pad markers
    /// and the scope's amplitude reference. The curve republishes
    /// only when the voicing tuple moves.
    pub(super) fn publish_static_viz(&mut self, voicing: Voicing) {
        let Voicing {
            character,
            stack,
            edge,
            bias,
            frame_bias,
            drive,
        } = voicing;
        // Transfer view: static skeleton of the curve stack (the
        // voicing filters and memory bend it dynamically).
        if (character as i32, stack as i32, edge, bias) != self.last_voicing {
            self.last_voicing = (character as i32, stack as i32, edge, bias);
            let cs_params = character_shape(character, edge as f64);
            let shape_single = cs_params.eval();
            let shape_pp = cs_params.eval_push_pull(0.7 - 0.45 * edge as f64, bias as f64 * 0.8);
            let kb = stage_b_knee(character, edge as f64);
            let post = shapes::knee(if stack == 2 { (1.5 + edge) as f64 } else { kb });
            let bf = frame_bias as f64;
            let span = CURVE_X_SPAN;
            let mut ys = [0.0f32; CURVE_POINTS];
            for (i, y) in ys.iter_mut().enumerate() {
                let u = -span + 2.0 * span * i as f64 / (CURVE_POINTS - 1) as f64;
                let a = if stack == 3 {
                    shape_pp(u)
                } else {
                    shape_single(u + bf)
                };
                *y = match stack {
                    0 => a,
                    _ => post(a),
                } as f32;
            }
            self.telemetry.publish_curve(&ys);
        }
        let nx = |u: f64| ((u + CURVE_X_SPAN) / (2.0 * CURVE_X_SPAN)).clamp(0.0, 1.0);
        self.taps.pad_rest_x.set(nx(frame_bias as f64));
        self.taps.pad_live_x.set(nx(frame_bias as f64));
        self.taps.pad_live_g.set(0.0);
        // Plugin-view reference: the shaper-domain amplitude a −9 dBFS
        // tone reaches at the current drive (harmonic-ladder input).
        self.sc_gain = 1.2 + 0.38 * drive;
        let viz_amp = if stack == 2 {
            self.sc_gain * 0.35
        } else {
            (1.0 + drive * 0.9) * 0.35
        };
        self.taps.viz_amp.set(viz_amp as f64);
    }
}

/// Forge's declared observation points: every wire the plugin
/// publishes, in ONE place. The stage taps (`tap0`…`tap5`, one per
/// chain slot in signal order) carry stable names — pill labels are
/// the face's business (`views::topology`), never the wire's.
#[derive(Taps)]
pub struct ForgeTaps {
    pub io_in: Tap,
    pub io_out: Tap,
    pub viz_amp: Tap,
    pub cpu_pct: Tap,
    pub pad_rest_x: Tap,
    pub pad_live_x: Tap,
    pub pad_live_g: Tap,
    pub tap0: Tap,
    pub tap1: Tap,
    pub tap2: Tap,
    pub tap3: Tap,
    pub tap4: Tap,
    pub tap5: Tap,
}

impl ForgeTaps {
    /// The per-stage taps' ids, in chain-slot order.
    pub const STAGES: [TapId; 6] = [Self::TAP0, Self::TAP1, Self::TAP2, Self::TAP3, Self::TAP4, Self::TAP5];

    /// The per-stage taps, in chain-slot order.
    pub fn stages(&self) -> [&Tap; 6] {
        [&self.tap0, &self.tap1, &self.tap2, &self.tap3, &self.tap4, &self.tap5]
    }
}
