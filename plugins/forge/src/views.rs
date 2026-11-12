//! Forge's own screens — the pictures its displays draw, each a plain
//! function of the frame (parameters, taps, the published curve and
//! the capture wire):
//!
//! * `transfer` — the curve stack's static skeleton with the live
//!   transfer cloud (shaper-domain input against the wet output);
//! * `harmonics` — H1…H8 of one cycle through the published curve at
//!   the engine's `viz_amp` reference (odd in phosphor, even in accent);
//! * `topology` — one pill per stage, lit by its tap level, labelled
//!   from the selectors' own step names (screamer renames stage 1,
//!   push-pull suffixes the character, SINGLE hides stage 2).

use stave::face::step_of;
use stave::prelude::*;
use stave::telemetry::CURVE_X_SPAN;

use crate::engine::ForgeTaps;
use crate::params::{stack, STACKS};

fn alpha(c: Color, a: f64) -> Color {
    Color::rgba(c.r, c.g, c.b, c.a * a)
}

/// The published curve sampled at `u` ∈ [−span, span] (linear between
/// points).
fn curve_at(ys: &[f32], u: f64) -> f64 {
    let n = ys.len();
    let t = ((u / CURVE_X_SPAN + 1.0) * 0.5).clamp(0.0, 1.0) * (n - 1) as f64;
    let i = (t as usize).min(n - 2);
    let fr = t - i as f64;
    f64::from(ys[i]) * (1.0 - fr) + f64::from(ys[i + 1]) * fr
}

/// The transfer: skeleton curve over the identity, the live cloud on top.
pub fn transfer(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    screens::curve(f, r, s);
    let mut a = vec![0f32; 1024];
    let mut b = vec![0f32; 1024];
    let n = f.controller.capture(0, &mut a).min(f.controller.capture(1, &mut b));
    let dot = alpha(f.theme.palette.accent, 0.55);
    for i in (0..n).step_by(4) {
        let x = r.x + r.w * ((f64::from(a[i]) / CURVE_X_SPAN + 1.0) * 0.5).clamp(0.0, 1.0);
        let y = r.y + r.h / 2.0 - f64::from(b[i]).clamp(-1.2, 1.2) / 1.2 * (r.h / 2.0 - 4.0);
        s.solid(Path::circle(pt(x, y), 1.1), dot);
    }
    let ty = &f.theme.typography;
    s.text_in(
        pt(r.x + r.w / 2.0, r.y + 14.0),
        "TRANSFER",
        ty.caption_size,
        alpha(f.theme.palette.phosphor, 0.6),
        ty.caption,
    );
}

/// The harmonic ladder: coherent Goertzel H1…H8 of one sine cycle at
/// the reference amplitude through the published curve.
pub fn harmonics(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    let th = &f.theme.palette;
    let ty = &f.theme.typography;
    let base = r.y + r.h - 18.0;
    let top = r.y + 22.0;
    s.text_in(
        pt(r.x + r.w / 2.0, r.y + 14.0),
        "HARMONICS",
        ty.caption_size,
        alpha(th.phosphor, 0.6),
        ty.caption,
    );
    s.stroke(
        Path::new()
            .move_to(pt(r.x + 8.0, base))
            .line_to(pt(r.x + r.w - 8.0, base)),
        Stroke::new(alpha(th.phosphor, 0.15), 1.0),
    );
    let ys = f.controller.curve();
    let mags = if ys.len() >= 2 {
        ladder(&ys, f.tap(ForgeTaps::VIZ_AMP).clamp(0.05, CURVE_X_SPAN))
    } else {
        [0.0; 8]
    };
    let h1 = mags[0].max(1e-9);
    let floor_db = -48.0;
    let slot = (r.w - 16.0) / 8.0;
    let bw = slot * 0.5;
    for (k, m) in mags.iter().enumerate() {
        let db = (20.0 * (m / h1).max(1e-9).log10()).max(floor_db);
        let norm = if ys.len() >= 2 {
            ((db - floor_db) / -floor_db).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let c = if k % 2 == 0 { th.phosphor } else { th.accent };
        let bx = r.x + 8.0 + k as f64 * slot + (slot - bw) / 2.0;
        let bh = (norm * (base - top)).max(1.5);
        s.solid(
            Path::rect(pt(bx - 2.0, base - bh - 2.0), bw + 4.0, bh + 2.0),
            alpha(c, 0.12),
        );
        s.solid(
            Path::rect(pt(bx, base - bh), bw, bh),
            alpha(c, if k == 0 { 0.5 } else { 0.85 }),
        );
        s.solid(Path::rect(pt(bx, base - bh), bw, 1.5), c);
        s.text_in(
            pt(bx + bw / 2.0, r.y + r.h - 6.0),
            format!("{}", k + 1),
            ty.caption_size,
            alpha(th.ink, 0.45),
            ty.value,
        );
    }
}

/// H1…H8 magnitudes of one cycle of `amp·sin` through the curve.
fn ladder(ys: &[f32], amp: f64) -> [f64; 8] {
    const N: usize = 256;
    let mut y = [0f64; N];
    let mut mean = 0.0;
    for (i, yi) in y.iter_mut().enumerate() {
        let x = amp * (core::f64::consts::TAU * i as f64 / N as f64).sin();
        *yi = curve_at(ys, x);
        mean += *yi;
    }
    mean /= N as f64;
    let mut mags = [0f64; 8];
    for (k, m) in mags.iter_mut().enumerate() {
        let w = core::f64::consts::TAU * (k + 1) as f64 / N as f64;
        let (mut s1, mut s2) = (0.0f64, 0.0f64);
        for &yi in &y {
            let s0 = (yi - mean) + 2.0 * w.cos() * s1 - s2;
            s2 = s1;
            s1 = s0;
        }
        *m = ((s1 - s2 * w.cos()).powi(2) + (s2 * w.sin()).powi(2)).sqrt() / (N as f64 * 0.5);
    }
    mags
}

/// The step name the `i`th declared bind sits on.
fn step_name(f: &Frame<'_>, i: usize) -> String {
    f.param(i)
        .and_then(|p| p.step_name(step_of(f.bound(i), p.step_count())))
        .unwrap_or_default()
        .into()
}

/// The stage strip: pills in signal order, each glowing with its tap.
/// Reads (in order) character, stack, filter, post.
pub fn topology(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    let th = &f.theme.palette;
    let ty = &f.theme.typography;
    let stack_i = step_of(f.bound(1), STACKS.len() as u32) as usize;
    let character = step_name(f, 0);
    let stage1 = match stack_i {
        stack::SCREAMER => String::from("SCREAM"),
        stack::PUSH_PULL => format!("{character} PP"),
        _ => character,
    };
    let stage2 = if stack_i == stack::SCREAMER {
        "POST"
    } else {
        "STAGE 2"
    };
    let [t0, t1, t2, t3, t4, t5] = ForgeTaps::STAGES;
    let pills: [(TapId, Option<String>); 6] = [
        (t0, Some(stage1)),
        (t1, Some(step_name(f, 2))),
        (t2, (stack_i != 0).then(|| String::from(stage2))),
        (t3, Some(step_name(f, 3))),
        (t4, Some(String::from("GRIT"))),
        (t5, Some(String::from("IRON"))),
    ];
    let shown: Vec<_> = pills
        .iter()
        .filter_map(|(t, l)| l.as_ref().map(|l| (*t, l)))
        .collect();
    let n = shown.len() as f64;
    let pad = 8.0;
    let arrow = 14.0;
    let pw = (r.w - 2.0 * pad - arrow * (n - 1.0)) / n;
    let ph = r.h - 10.0;
    let cy = r.y + r.h / 2.0;
    for (i, (tap, label)) in shown.iter().enumerate() {
        let x = r.x + pad + i as f64 * (pw + arrow);
        let level = f.tap(*tap).abs().clamp(0.0, 1.0);
        let glow = level.sqrt();
        let pill = Rect {
            x,
            y: cy - ph / 2.0,
            w: pw,
            h: ph,
        };
        s.solid(
            rrect_path(pill, ph / 2.0),
            alpha(th.phosphor, 0.06 + 0.30 * glow),
        );
        s.stroke(
            rrect_path(pill, ph / 2.0),
            Stroke::new(alpha(th.phosphor, 0.25 + 0.6 * glow), 1.0),
        );
        s.text_in(
            pt(x + pw / 2.0, cy + 3.0),
            label.as_str(),
            ty.caption_size,
            alpha(th.phosphor, 0.55 + 0.45 * glow),
            ty.caption,
        );
        if i + 1 < shown.len() {
            let ax = x + pw + 3.0;
            s.stroke(
                Path::new()
                    .move_to(pt(ax, cy))
                    .line_to(pt(ax + arrow - 6.0, cy)),
                Stroke::new(alpha(th.phosphor, 0.3), 1.0),
            );
            s.solid(
                Path::new()
                    .move_to(pt(ax + arrow - 6.0, cy - 2.5))
                    .line_to(pt(ax + arrow - 3.0, cy))
                    .line_to(pt(ax + arrow - 6.0, cy + 2.5))
                    .close(),
                alpha(th.phosphor, 0.3),
            );
        }
    }
}
