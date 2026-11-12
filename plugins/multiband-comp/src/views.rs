//! Multiband's screens, each a plain function of the frame:
//!
//! * `spectrum` — the output spectrum (capture channel 1) with the
//!   three bands tinted between the crossover splits, each band's
//!   threshold as a rule and its gain reduction hanging from the top;
//! * `reduction` — the three bands' GR as hanging bars, with readouts;
//! * `transfer_*` — one band's static gain curve (the engine's own gain
//!   computer, with the style's knee when the knob is at zero) and the
//!   live reduction as a dot riding it.

extern crate alloc;
use alloc::format;
use std::sync::LazyLock;

use stave_core::face::prelude::*;
use stave_core::face::step_of;
use stave_core::params::{BindParam, ParamEnum};
use stave_ui::scene::TextAlign;

use crate::engine::dynamics::band_reduction_db;
use crate::params::{MbParams, BANDS, GR_PROBES};
use crate::style::Style;

static P: LazyLock<alloc::sync::Arc<MbParams>> = LazyLock::new(MbParams::new);

/// Each band's tint on the glass: amber, phosphor green, ice.
const TINTS: [Color; BANDS] = [
    Color::rgba(1.0, 0.72, 0.32, 1.0),
    Color::rgba(0.66, 0.92, 0.52, 1.0),
    Color::rgba(0.55, 0.85, 0.92, 1.0),
];
const NAMES: [&str; BANDS] = ["LOW", "MID", "HIGH"];

const F_LO: f64 = 20.0;
const F_HI: f64 = 20_000.0;

fn alpha(c: Color, a: f64) -> Color {
    Color::rgba(c.r, c.g, c.b, c.a * a)
}

fn x_of(r: Rect, hz: f64) -> f64 {
    r.x + r.w * ((hz / F_LO).ln() / (F_HI / F_LO).ln()).clamp(0.0, 1.0)
}

fn plain(f: &Frame<'_>, b: Bind, range: stave_core::params::Range) -> f64 {
    range.denormalize(f.value(b))
}

pub fn spectrum(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    let p = &**P;
    let sr = f.probe("sample_rate");
    let sr = if sr > 1000.0 { sr } else { 48_000.0 };
    let splits = [
        plain(f, p.xover_low.bind(), p.xover_low.range()),
        plain(f, p.xover_high.bind(), p.xover_high.range()),
    ];
    let edges = [r.x, x_of(r, splits[0]), x_of(r, splits[1]), r.x + r.w];
    // Band washes under everything.
    for (i, tint) in TINTS.iter().enumerate() {
        s.solid(
            Path::rect(pt(edges[i], r.y), edges[i + 1] - edges[i], r.h),
            alpha(*tint, 0.05),
        );
    }
    screens::spectrum(f, r, s, 1, sr);
    let ty = &f.theme.type_;
    for (i, tint) in TINTS.iter().enumerate() {
        let (x0, x1) = (edges[i], edges[i + 1]);
        // Threshold: −60…0 dB on the spectrum's −90…0 scale.
        let th = plain(f, p.bands[i].threshold.bind(), p.bands[i].threshold.range());
        let y = r.y + r.h - 3.0 - ((th + 90.0) / 90.0).clamp(0.0, 1.0) * (r.h - 8.0);
        s.stroke(
            Path::new()
                .move_to(pt(x0 + 3.0, y))
                .line_to(pt(x1 - 3.0, y)),
            Stroke::new(alpha(*tint, 0.7), 1.0),
        );
        // Gain reduction hangs from the top (24 dB = full height/2).
        let gr = (-f.probe(GR_PROBES[i])).clamp(0.0, 24.0);
        let h = gr / 24.0 * r.h * 0.5;
        if h > 0.5 {
            s.solid(
                Path::rect(pt(x0 + 2.0, r.y), x1 - x0 - 4.0, h),
                alpha(*tint, 0.28),
            );
            s.solid(
                Path::rect(pt(x0 + 2.0, r.y + h - 1.5), x1 - x0 - 4.0, 1.5),
                *tint,
            );
        }
        s.text_in(
            pt((x0 + x1) / 2.0, r.y + r.h - 6.0),
            NAMES[i],
            ty.caption_size,
            alpha(*tint, 0.75),
            ty.caption,
        );
    }
    for x in &edges[1..3] {
        s.stroke(
            Path::new()
                .move_to(pt(*x, r.y + 2.0))
                .line_to(pt(*x, r.y + r.h - 2.0)),
            Stroke::new(alpha(f.theme.ink, 0.5), 1.0),
        );
    }
    for (x, hz) in edges[1..3].iter().zip(splits) {
        let label = if hz >= 1000.0 {
            format!("{:.1}k", hz / 1000.0)
        } else {
            format!("{hz:.0}")
        };
        s.text_in(
            pt(*x + 4.0, r.y + 12.0),
            label,
            ty.caption_size,
            alpha(f.theme.ink, 0.7),
            ty.value.align(TextAlign::Left),
        );
    }
}

pub fn reduction(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    let ty = &f.theme.type_;
    let slot = r.w / BANDS as f64;
    let (top, bottom) = (r.y + 8.0, r.y + r.h - 28.0);
    for db in [6.0, 12.0, 18.0] {
        let y = top + (bottom - top) * db / 24.0;
        s.stroke(
            Path::new()
                .move_to(pt(r.x + 4.0, y))
                .line_to(pt(r.x + r.w - 4.0, y)),
            Stroke::new(alpha(f.theme.phosphor, 0.08), 1.0),
        );
    }
    for (i, tint) in TINTS.iter().enumerate() {
        let gr = (-f.probe(GR_PROBES[i])).clamp(0.0, 24.0);
        let cx = r.x + slot * (i as f64 + 0.5);
        let bw = slot * 0.42;
        s.solid(
            Path::rect(pt(cx - bw / 2.0, top), bw, bottom - top),
            alpha(*tint, 0.06),
        );
        let h = gr / 24.0 * (bottom - top);
        if h > 0.3 {
            s.solid(Path::rect(pt(cx - bw / 2.0, top), bw, h), alpha(*tint, 0.8));
        }
        s.text_in(
            pt(cx, r.y + r.h - 16.0),
            if gr >= 0.05 {
                format!("-{gr:.1}")
            } else {
                format!("{:.1}", 0.0)
            },
            ty.caption_size,
            alpha(*tint, 0.9),
            ty.value,
        );
        s.text_in(
            pt(cx, r.y + r.h - 5.0),
            NAMES[i],
            ty.caption_size,
            alpha(f.theme.ink, 0.5),
            ty.caption,
        );
    }
}

pub fn transfer_lo(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    transfer(f, r, s, 0);
}
pub fn transfer_mid(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    transfer(f, r, s, 1);
}
pub fn transfer_hi(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    transfer(f, r, s, 2);
}

/// Input −60…0 dB across, output −60…+12 dB up.
fn transfer(f: &Frame<'_>, r: Rect, s: &mut Scene, band: usize) {
    let b = &P.bands[band];
    let tint = TINTS[band];
    screens::grid(f, r, s, 6, 4);
    let threshold = plain(f, b.threshold.bind(), b.threshold.range()) as f32;
    let ratio = plain(f, b.ratio.bind(), b.ratio.range()) as f32;
    let knee_knob = plain(f, b.knee.bind(), b.knee.range()) as f32;
    let makeup = plain(f, b.makeup.bind(), b.makeup.range());
    let style = Style::from_index(step_of(f.value(b.style.bind()), Style::COUNT));
    let knee = if knee_knob > 0.0 {
        knee_knob
    } else {
        style.character().knee_db
    };
    let (lo, hi) = (-60.0f64, 12.0f64);
    let map = |i: f64, o: f64| {
        pt(
            r.x + r.w * (i - lo) / (0.0 - lo),
            r.y + r.h - r.h * ((o - lo) / (hi - lo)).clamp(0.0, 1.0),
        )
    };
    s.stroke(
        Path::new().move_to(map(lo, lo)).line_to(map(0.0, 0.0)),
        Stroke::new(alpha(f.theme.phosphor, 0.18), 1.0),
    );
    let out = |i: f64| {
        i - f64::from(band_reduction_db(
            stave_dsp::db_to_amp(i as f32),
            threshold,
            ratio,
            knee,
        )) + makeup
    };
    let pts: alloc::vec::Vec<_> = (0..=60)
        .map(|k| {
            let i = lo + (0.0 - lo) * f64::from(k) / 60.0;
            map(i, out(i))
        })
        .collect();
    s.stroke(Path::polyline(&pts), Stroke::round(alpha(tint, 0.18), 5.0));
    s.stroke(Path::polyline(&pts), Stroke::round(tint, 1.5));
    // The live operating point: the input level that pulls this much.
    let gr = -f.probe(GR_PROBES[band]);
    if gr > 0.05 {
        let slope = 1.0 - 1.0 / f64::from(ratio);
        let i = (f64::from(threshold) + gr / slope.max(1e-3)).min(0.0);
        s.solid(Path::circle(map(i, out(i)), 3.0), f.theme.ink);
    }
    let ty = &f.theme.type_;
    s.text_in(
        pt(r.x + 6.0, r.y + 12.0),
        format!("{} · {:.1}:1", NAMES[band], ratio),
        ty.caption_size,
        alpha(tint, 0.8),
        ty.caption.align(TextAlign::Left),
    );
}
