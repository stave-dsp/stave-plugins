//! The ribbon face — an oxblood crinkle case whose window shows the
//! machine running: the tape loop round two aluminium rollers and along
//! the head block (erase, record, three playback heads, capstan). It
//! moves at the motor's real speed (the `tape_loop` tap), its splice
//! comes round once a loop, the rollers turn with it, and lamps glow
//! under the live heads. Under the window: the ECHO (RATE, INTENSITY,
//! HEADS), the TAPE (DRIVE, WEAR, ERASE), the OUTPUT (SPRING, MIX), and
//! the BYPASS / BRAKE footswitches.

use core::f64::consts::{PI, TAU};

use stave::face::lamp;
use stave::prelude::*;

use crate::params::RibbonParams;
use crate::RibbonTaps;

pub fn face(p: &RibbonParams) -> Face {
    let theme = Theme::oxblood();
    let gap = theme.space(4);
    let header = row(vec![
        Title::new("RIBBON").tagline("tape echo · spring · erase head").node(),
        spacer(),
        Presets { width: 184.0 }.node(),
    ])
    .gap(theme.space(3))
    .align(Align::Start);
    let window = Display::new(640.0, 168.0, draw_tape)
        .taps(&[RibbonTaps::TAPE_LOOP, RibbonTaps::TAPE_SPEED])
        .reads(&[p.heads.bind()])
        .node();
    let echo = Plate::titled("ECHO").around(
        row(vec![
            Knob::new(p.rate.bind()).large().node(),
            Knob::new(p.intensity.bind()).large().node(),
            Selector::new(p.heads.bind()).node(),
        ])
        .gap(gap)
        .align(Align::Center),
    );
    let output = Plate::titled("OUTPUT").around(
        row(vec![Knob::new(p.spring.bind()).large().node(), Knob::new(p.mix.bind()).large().node()])
            .gap(gap)
            .align(Align::Center),
    );
    let tape = Plate::titled("TAPE").around(
        row(vec![Knob::new(p.drive.bind()).node(), Knob::new(p.wear.bind()).node(), Knob::new(p.erase.bind()).node()])
            .gap(gap)
            .align(Align::Center),
    );
    let lower = row(vec![
        Footswitch::bypass(p.bypass.bind()).node(),
        tape,
        Footswitch::new(p.brake.bind()).label("BRAKE").node(),
    ])
    .gap(gap)
    .align(Align::Center);
    let body = column(vec![header, window, row(vec![echo, output]).gap(gap).align(Align::Stretch), lower])
        .gap(theme.space(4))
        .align(Align::Stretch);
    Face::new(theme, Panel::default().around(body))
}

// ---------------------------------------------------------- the machine

/// Which playback heads each HEADS step makes live.
const HEAD_SETS: [[bool; 3]; 7] = [
    [true, false, false],
    [false, true, false],
    [false, false, true],
    [true, true, false],
    [false, true, true],
    [true, false, true],
    [true, true, true],
];
/// Heads along the bottom run (fraction of the screen's width), legend.
const HEADS: [(f64, &str); 5] = [(0.31, "E"), (0.38, "R"), (0.48, "1"), (0.58, "2"), (0.68, "3")];
const CAPSTAN: f64 = 0.78;

/// The tape path in one screen: two rollers joined by straight runs.
/// The ribbon travels left→right along the BOTTOM run (past the heads),
/// up round the right roller, back along the top.
struct Deck {
    l: Xy,
    r: Xy,
    rad: f64,
}

/// A point on the deck (the scene's own point type is not on the
/// author surface; `pt(p.x, p.y)` draws it).
#[derive(Copy, Clone)]
struct Xy {
    x: f64,
    y: f64,
}

fn xy(x: f64, y: f64) -> Xy {
    Xy { x, y }
}

impl Deck {
    fn of(s: Rect) -> Self {
        let rad = s.h * 0.24;
        let cy = s.y + s.h * 0.42;
        Deck { l: xy(s.x + s.w * 0.13, cy), r: xy(s.x + s.w * 0.87, cy), rad }
    }
    fn run(&self) -> f64 {
        self.r.x - self.l.x
    }
    fn len(&self) -> f64 {
        2.0 * self.run() + TAU * self.rad
    }
    /// A point `s` points along the path, and the direction there.
    fn at(&self, s: f64) -> (Xy, (f64, f64)) {
        let (run, arc, rad) = (self.run(), PI * self.rad, self.rad);
        let s = s.rem_euclid(self.len());
        if s < run {
            (xy(self.l.x + s, self.l.y + rad), (1.0, 0.0))
        } else if s < run + arc {
            let a = PI / 2.0 - (s - run) / rad;
            (xy(self.r.x + rad * a.cos(), self.r.y + rad * a.sin()), (a.sin(), -a.cos()))
        } else if s < 2.0 * run + arc {
            (xy(self.r.x - (s - run - arc), self.r.y - rad), (-1.0, 0.0))
        } else {
            let a = -PI / 2.0 - (s - 2.0 * run - arc) / rad;
            (xy(self.l.x + rad * a.cos(), self.l.y + rad * a.sin()), (a.sin(), -a.cos()))
        }
    }
}

fn hash(i: u32) -> f64 {
    let mut h = i.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    f64::from(h) / 4_294_967_296.0
}

fn grey(v: f64, a: f64) -> Color {
    Color::rgba(v, v, v * 1.02, a)
}

/// A machined part lit from the upper left: a vertical sweep of greys
/// over `y0..y1`, filling the rounded rect `r`.
fn metal(s: &mut Scene, r: Rect, radius: f64, ys: (f64, f64), hi: f64, lo: f64) {
    vertical_gradient(s, r, radius, ys, &[(0.0, grey(hi, 1.0)), (0.55, grey((hi + lo) / 2.0, 1.0)), (1.0, grey(lo, 1.0))]);
}

/// A machined disc of radius `rad` at (`x`, `y`), its sweep over `y0..y1`.
fn metal_disc(s: &mut Scene, (x, y): (f64, f64), rad: f64, ys: (f64, f64), hi: f64, lo: f64) {
    metal(s, Rect { x: x - rad, y: y - rad, w: 2.0 * rad, h: 2.0 * rad }, rad, ys, hi, lo);
}

fn draw_tape(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    let d = Deck::of(r);
    let phase = Some(f.tap(RibbonTaps::TAPE_LOOP)).filter(|v| v.is_finite()).unwrap_or(0.0);
    let travel = phase * d.len();
    let run_y = d.l.y + d.rad;

    // The deck: dark brushed steel.
    vertical_gradient(s, r, 7.0, (r.y, r.y + r.h), &[(0.0, grey(0.13, 1.0)), (1.0, grey(0.07, 1.0))]);
    for i in 0..60u32 {
        let y = r.y + 6.0 + (r.h - 12.0) * hash(i);
        let x0 = r.x + 8.0 + (r.w - 16.0) * hash(i + 99) * 0.7;
        let len = 40.0 + 160.0 * hash(i + 199);
        let x1 = (x0 + len).min(r.x + r.w - 8.0);
        s.stroke(Path::new().move_to(pt(x0, y)).line_to(pt(x1, y)), Stroke::new(Color::rgba(1.0, 1.0, 1.0, 0.025), 0.6));
    }

    // Rollers: shadow, a machined aluminium disc, a hub, spokes turned by
    // the tape passing over them.
    for c in [d.l, d.r] {
        let c = pt(c.x, c.y);
        s.solid(Path::circle(pt(c.x + 4.0, c.y + 6.0), d.rad), Color::rgba(0.0, 0.0, 0.0, 0.45));
        metal_disc(s, (c.x, c.y), d.rad - 2.0, (c.y - d.rad, c.y + d.rad), 0.72, 0.32);
        s.stroke(Path::circle(c, d.rad - 6.0), Stroke::new(grey(0.2, 0.5), 1.0));
        let a0 = travel / d.rad;
        for k in 0..3 {
            let a = a0 + TAU * f64::from(k) / 3.0;
            let (ca, sa) = (a.cos(), a.sin());
            let (r0, r1) = (8.0, d.rad - 9.0);
            s.stroke(
                Path::new().move_to(pt(c.x + ca * r0, c.y + sa * r0)).line_to(pt(c.x + ca * r1, c.y + sa * r1)),
                Stroke::round(Color::rgba(0.12, 0.12, 0.13, 0.85), 4.0),
            );
            s.stroke(
                Path::new()
                    .move_to(pt(c.x + ca * r0 - 0.8, c.y + sa * r0 - 0.8))
                    .line_to(pt(c.x + ca * r1 - 0.8, c.y + sa * r1 - 0.8)),
                Stroke::round(Color::rgba(1.0, 1.0, 1.0, 0.2), 1.0),
            );
        }
        metal_disc(s, (c.x, c.y), 7.0, (c.y - 7.0, c.y + 7.0), 0.92, 0.4);
        s.solid(Path::circle(c, 2.0), grey(0.1, 1.0));
    }

    // The head block: chrome heads with their gap, the capstan and its
    // rubber pinch roller.
    for (k, &(fx, _)) in HEADS.iter().enumerate() {
        let x = r.x + r.w * fx;
        let head = Rect { x: x - 10.0, y: run_y - 3.0, w: 20.0, h: 25.0 };
        s.solid(rrect_path(Rect { x: head.x + 3.0, y: head.y + 5.0, ..head }, 3.0), Color::rgba(0.0, 0.0, 0.0, 0.45));
        let (hi, lo) = if k == 0 { (0.78, 0.4) } else { (0.92, 0.45) };
        metal(s, head, 3.0, (head.y, head.y + head.h), hi, lo);
        s.stroke(Path::new().move_to(pt(x, head.y + 1.0)).line_to(pt(x, head.y + 9.0)), Stroke::new(grey(0.03, 1.0), 0.9));
    }
    let cx = r.x + r.w * CAPSTAN;
    s.solid(Path::circle(pt(cx + 3.0, run_y + 16.0), 11.0), Color::rgba(0.0, 0.0, 0.0, 0.45));
    s.solid(Path::circle(pt(cx, run_y + 12.0), 11.0), grey(0.05, 1.0));
    s.stroke(Path::circle(pt(cx, run_y + 12.0), 10.0), Stroke::new(grey(0.2, 0.6), 1.0));
    metal_disc(s, (cx, run_y + 12.0), 3.5, (run_y + 8.0, run_y + 16.0), 0.85, 0.4);
    metal_disc(s, (cx, run_y - 7.0), 4.5, (run_y - 11.0, run_y - 3.0), 0.95, 0.45);

    // The ribbon: shadow, oxide brown, a satin sheen.
    let pts: Vec<_> = (0..=200)
        .map(|i| {
            let p = d.at(d.len() * f64::from(i) / 200.0).0;
            pt(p.x, p.y)
        })
        .collect();
    let shadow: Vec<_> = pts.iter().map(|p| pt(p.x + 2.0, p.y + 3.0)).collect();
    s.stroke(Path::polyline(&shadow), Stroke::round(Color::rgba(0.0, 0.0, 0.0, 0.5), 9.0));
    s.stroke(Path::polyline(&pts), Stroke::round(Color::rgba(0.25, 0.14, 0.08, 1.0), 7.0));
    s.stroke(Path::polyline(&pts), Stroke::round(Color::rgba(0.55, 0.36, 0.22, 0.35), 2.2));
    // Oxide streaks fixed on the tape (irregular, so they never strobe).
    let l = d.len();
    for i in 0..64u32 {
        let at = hash(i) * l + travel;
        let len = 4.0 + 10.0 * hash(i + 1000);
        let (p0, dir) = d.at(at);
        let (p1, _) = d.at(at + len);
        let off = (hash(i + 2000) - 0.5) * 3.5;
        let n = (-dir.1, dir.0);
        s.stroke(
            Path::new().move_to(pt(p0.x + n.0 * off, p0.y + n.1 * off)).line_to(pt(p1.x + n.0 * off, p1.y + n.1 * off)),
            Stroke::new(Color::rgba(0.12, 0.06, 0.03, 0.45 + 0.3 * hash(i + 3000)), 0.8),
        );
    }
    // The splice: a strip of pale leader tape, once round the loop.
    let splice: Vec<_> = (0..=8)
        .map(|k| {
            let p = d.at(travel + 20.0 * f64::from(k) / 8.0).0;
            pt(p.x, p.y)
        })
        .collect();
    s.stroke(Path::polyline(&splice), Stroke::new(Color::rgba(0.88, 0.85, 0.76, 0.95), 7.0));

    // Legends under the heads, lamps under the live playback heads.
    let ty = &f.theme.typography;
    let ink = Color::rgba(0.78, 0.78, 0.76, 0.75);
    // HEADS: the one bind the window declares (`reads` in `face`).
    let set = HEAD_SETS[(f.bound(0) * 6.0).round().clamp(0.0, 6.0) as usize];
    let lamp_y = r.y + r.h - 16.0;
    for (k, &(fx, label)) in HEADS.iter().enumerate() {
        let x = r.x + r.w * fx;
        s.text_in(pt(x, lamp_y - 9.0), label, ty.caption_size, ink, ty.caption);
        if k >= 2 {
            s.solid(Path::circle(pt(x, lamp_y + 2.0), 3.0), grey(0.04, 1.0));
            if set[k - 2] {
                lamp(f, s, pt(x, lamp_y + 2.0));
            }
        }
    }
    // The motor's speed, read off in the corner like a deck's counter.
    let speed = Some(f.tap(RibbonTaps::TAPE_SPEED)).filter(|v| v.is_finite()).unwrap_or(0.0);
    // Set flush to the corners by half their width (the author surface
    // has no `TextAlign`): the value font is mono, ~0.6 em a figure; the
    // caption tracked caps, ~0.78 em a letter.
    let ph = f.theme.palette.phosphor;
    let counter = format!("{:.2}×", speed.max(0.0));
    let counter_half = 0.5 * counter.chars().count() as f64 * ty.value_size * 0.6;
    s.text_in(pt(r.x + r.w - 12.0 - counter_half, r.y + 18.0), counter, ty.value_size, ph, ty.value);
    let loop_half = 0.5 * 4.0 * ty.caption_size * 0.78;
    s.text_in(pt(r.x + 12.0 + loop_half, r.y + 18.0), "LOOP", ty.caption_size, ph, ty.caption);
}

/// A vertical linear gradient (`stops` over `y0..y1`) filling the
/// rounded rect `r` (corner `radius`; a square with half-side radius is
/// a circle). The face's author surface has no gradient brush, so it is
/// painted as 1 pt bands of solid colour, each overlapping the next by a
/// band so no seam shows.
fn vertical_gradient(s: &mut Scene, r: Rect, radius: f64, (y0, y1): (f64, f64), stops: &[(f64, Color)]) {
    const BAND: f64 = 1.0;
    let rad = radius.min(r.w / 2.0).min(r.h / 2.0);
    // The outline's horizontal inset at height y.
    let inset = |y: f64| {
        let dy = (r.y + rad - y).max(y - (r.y + r.h - rad)).clamp(0.0, rad);
        rad - (rad * rad - dy * dy).sqrt()
    };
    let colour = |t: f64| {
        let t = t.clamp(0.0, 1.0);
        let i = stops.iter().position(|(at, _)| *at >= t).unwrap_or(stops.len() - 1);
        if i == 0 {
            return stops[0].1;
        }
        let ((ta, a), (tb, b)) = (stops[i - 1], stops[i]);
        let u = if tb > ta { (t - ta) / (tb - ta) } else { 1.0 };
        let mix = |x: f64, y: f64| x + (y - x) * u;
        Color::rgba(mix(a.r, b.r), mix(a.g, b.g), mix(a.b, b.b), mix(a.a, b.a))
    };
    let bands = (r.h / BAND).ceil().max(1.0) as usize;
    for i in 0..bands {
        let top = r.y + i as f64 * BAND;
        let bot = (top + 2.0 * BAND).min(r.y + r.h);
        let ys = [top, (top + bot) / 2.0, bot];
        let mut path = Path::new().move_to(pt(r.x + inset(top), top));
        for &y in &ys[1..] {
            path = path.line_to(pt(r.x + inset(y), y));
        }
        for &y in ys.iter().rev() {
            path = path.line_to(pt(r.x + r.w - inset(y), y));
        }
        let mid = (top + BAND / 2.0).min(r.y + r.h);
        s.solid(path.close(), colour((mid - y0) / (y1 - y0)));
    }
}
