//! The heirloom face — a cream pedal: the reverse machine on top
//! (TIME / REPEATS / MIX, large), the worn record under it (WOBBLE /
//! RPM / AGE), and between the footswitches the record itself, rising
//! like a sun behind glass, its label turning at the engine's platter
//! angle (the `spin` probe — wobble lurches, HOLD sags, bypass brakes).

use core::f64::consts::{PI, TAU};

use stave_core::face::prelude::*;
use stave_core::params::BindParam;
use stave_ui::scene::{Pt, TextAlign};

use crate::clip;
use crate::params::HeirloomParams;

pub fn face(p: &HeirloomParams) -> Face {
    let theme = Theme::cream();
    let gap = theme.space(4);
    let header = row(vec![
        Title::new("HEIRLOOM").tagline("reverse machines · played off worn media").node(),
        spacer(),
        Stepper::new(p.sync.bind()).label("SYNC").width(112.0).node(),
        Presets { width: 176.0 }.node(),
    ])
    .gap(theme.space(3))
    .align(Align::Start);
    let machine = Plate::titled("REVERSE").around(
        row(vec![
            Toggle::new(p.machine.bind()).node(),
            Knob::new(p.time.bind()).large().node(),
            Knob::new(p.repeats.bind()).large().node(),
            Knob::new(p.mix.bind()).large().node(),
        ])
        .gap(gap)
        .align(Align::Center),
    );
    let record = Plate::titled("RECORD").around(
        row(vec![
            Toggle::new(p.warp.bind()).node(),
            Knob::new(p.wobble.bind()).node(),
            Knob::new(p.rpm.bind()).node(),
            Knob::new(p.age.bind()).node(),
            Toggle::new(p.media.bind()).node(),
        ])
        .gap(gap)
        .align(Align::Center),
    );
    let platter = Display::new(340.0, 150.0, draw_record).probes(&["spin"]).node();
    let feet = row(vec![
        Footswitch::bypass(p.bypass.bind()).node(),
        spacer(),
        platter,
        spacer(),
        Footswitch::new(p.hold.bind()).label("HOLD").node(),
    ])
    .align(Align::End);
    let body = column(vec![header, machine, record, feet]).gap(theme.space(4)).align(Align::Stretch);
    Face::new(theme, Panel::default().around(body))
}

fn rgb(h: u32, a: f64) -> Color {
    let ch = |s: u32| f64::from((h >> s) & 0xFF) / 255.0;
    Color::rgba(ch(16), ch(8), ch(0), a)
}

/// Points round `c` at radius `rad(a)` from angle `a0` to `a1`.
fn arc_pts(c: Pt, a0: f64, a1: f64, n: usize, rad: impl Fn(f64) -> f64) -> Vec<Pt> {
    (0..=n)
        .map(|i| {
            let a = a0 + (a1 - a0) * i as f64 / n as f64;
            let r = rad(a);
            pt(c.x + r * a.cos(), c.y + r * a.sin())
        })
        .collect()
}

fn hash(i: u32) -> f64 {
    let mut h = i.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    f64::from(h) / 4_294_967_296.0
}

/// The record: a warped black disc rising from the screen's lower edge,
/// its grooves catching the studio light in two still bands (light
/// doesn't turn), and the printed label turning by `spin`.
fn draw_record(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    let spin = Some(f.probe("spin")).filter(|a| a.is_finite()).unwrap_or(0.0);
    let c = pt(r.x + r.w / 2.0, r.y + r.h - 6.0);
    let rad = r.h - 20.0;
    let lab = rad * 0.37;
    let fill = |s: &mut Scene, pts: &[Pt], col: Color| {
        if let Some(p) = clip::polygon(pts, r) {
            s.solid(p, col);
        }
    };
    let line = |s: &mut Scene, pts: &[Pt], st: Stroke| {
        for p in clip::polyline(pts, r) {
            s.stroke(p, st);
        }
    };

    // The disc and its warped edge.
    let warp = |a: f64| rad * (1.0 + 0.012 * (3.0 * a + 0.7).sin() + 0.005 * (5.0 * a - 1.1).sin());
    let disc = arc_pts(c, 0.0, TAU, 180, warp);
    fill(s, &disc, rgb(0x0A0A0C, 1.0));
    line(s, &disc, Stroke::new(Color::rgba(1.0, 1.0, 1.0, 0.12), 1.0));

    // Grooves: fine rings, a few track gaps.
    let mut gr = lab + 7.0;
    let mut k = 0u32;
    while gr < rad - 4.0 {
        let gap = [0.56, 0.7, 0.84].iter().any(|g| (gr / rad - g).abs() < 0.014);
        if !gap {
            let a = 0.03 + 0.04 * hash(k);
            line(s, &clip::circle(c, gr, 120), Stroke::new(Color::rgba(1.0, 1.0, 1.0, a), 0.6));
        }
        gr += 2.2;
        k += 1;
    }

    // The groove sheen: the light's two still bands across the disc,
    // layered soft, with the print's one cool note (a teal glint).
    let az = -2.2_f64;
    for band in [az, az + PI] {
        for (w, a) in [(0.42, 0.018), (0.32, 0.022), (0.23, 0.026), (0.15, 0.03), (0.08, 0.035)] {
            let mut pts = arc_pts(c, band - w * 0.5, band + w * 0.5, 10, |_| lab + 5.0);
            pts.extend(arc_pts(c, band + w, band - w, 20, |_| rad - 5.0));
            fill(s, &pts, Color::rgba(0.92, 0.94, 1.0, a));
        }
        line(s, &arc_pts(c, band - 0.15, band + 0.15, 16, |_| rad * 0.74), Stroke::new(rgb(0x4FB3AA, 0.4), 2.0));
    }

    // The label: two printed halves split on a diameter, a cream rim, a
    // ring of 24 dots and a spoke — all turned by the platter.
    let mut half = arc_pts(c, spin, spin + PI, 48, |_| lab);
    fill(s, &half, rgb(0xD9A23A, 1.0));
    half = arc_pts(c, spin + PI, spin + TAU, 48, |_| lab);
    fill(s, &half, rgb(0xB4532A, 1.0));
    line(s, &clip::circle(c, lab - 5.0, 90), Stroke::new(rgb(0xEADFC6, 1.0), 1.8));
    for i in 0..24 {
        let a = spin + TAU * f64::from(i) / 24.0;
        let d = lab - 12.0;
        let p = pt(c.x + d * a.cos(), c.y + d * a.sin());
        if p.y < r.y + r.h - 1.5 {
            s.solid(Path::circle(p, 1.4), rgb(0x3A1C10, 1.0));
        }
    }
    let sa = spin + 0.9;
    let spoke = [pt(c.x + 10.0 * sa.cos(), c.y + 10.0 * sa.sin()), pt(c.x + (lab - 19.0) * sa.cos(), c.y + (lab - 19.0) * sa.sin())];
    line(s, &spoke, Stroke::new(rgb(0xEADFC6, 1.0), 2.4));
    // A soft light falling across the paper, then the spindle.
    let mut lit = arc_pts(c, az - 1.2, az + 1.2, 24, |_| lab);
    lit.push(c);
    fill(s, &lit, Color::rgba(1.0, 0.97, 0.9, 0.08));
    fill(s, &clip::circle(c, 4.0, 24), rgb(0x0A0A0C, 1.0));
    fill(s, &clip::circle(c, 2.0, 16), rgb(0x9A968E, 1.0));

    // The tonearm, resting in the outer grooves from the right.
    let pivot = pt(r.x + r.w - 28.0, r.y + 28.0);
    let ta = -0.72_f64;
    let tip = pt(c.x + rad * 0.88 * ta.cos(), c.y + rad * 0.88 * ta.sin());
    let shadow = [pt(pivot.x + 2.0, pivot.y + 4.0), pt(tip.x + 2.0, tip.y + 4.0)];
    s.stroke(Path::polyline(&shadow), Stroke::round(Color::rgba(0.0, 0.0, 0.0, 0.5), 3.6));
    s.stroke(Path::polyline(&[pivot, tip]), Stroke::round(rgb(0xC9C5BC, 1.0), 2.6));
    s.stroke(Path::polyline(&[pivot, tip]), Stroke::round(Color::rgba(1.0, 1.0, 1.0, 0.4), 0.8));
    let (dx, dy) = (tip.x - pivot.x, tip.y - pivot.y);
    let len = dx.hypot(dy);
    let (ux, uy) = (dx / len, dy / len);
    let head = Path::new()
        .move_to(pt(tip.x - uy * 5.0, tip.y + ux * 5.0))
        .line_to(pt(tip.x + ux * 12.0 - uy * 4.0, tip.y + uy * 12.0 + ux * 4.0))
        .line_to(pt(tip.x + ux * 12.0 + uy * 4.0, tip.y + uy * 12.0 - ux * 4.0))
        .line_to(pt(tip.x + uy * 5.0, tip.y - ux * 5.0))
        .close();
    s.solid(head, rgb(0x2A2826, 1.0));
    s.solid(Path::circle(pivot, 12.0), rgb(0x2E2B28, 1.0));
    s.solid(Path::circle(pivot, 9.0), rgb(0xB8B3AA, 1.0));
    s.solid(Path::circle(pivot, 2.5), rgb(0x2A2826, 1.0));

    // 33⅓ in the screen's phosphor.
    let ty = &f.theme.type_;
    s.text_in(pt(r.x + 12.0, r.y + 18.0), "33⅓", ty.caption_size, f.theme.phosphor, ty.caption.align(TextAlign::Left));
}
