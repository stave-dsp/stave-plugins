//! The revenant face — a hammertone laboratory instrument. Two screens
//! behind glass: a PORTHOLE onto the living network (24 of its neurons,
//! glowing cyan as they fire positive and violet negative, joined by
//! filaments that carry light when both ends are active) and an analog
//! SURPRISE meter whose needle swings with how wrong its last guesses
//! were. Under them the mind (LEARN / MEMORY / CHAOS / HORIZON) and the
//! voice (DREAM / MIX), and the BYPASS (red lamp) / SLEEP (blue lamp) footswitches.

use core::f64::consts::TAU;
use std::sync::OnceLock;

use stave::face::material;
use stave::face::part::sprite_key;
use stave::face::photo::{hex, mix, mul, Rgb};
use stave::prelude::*;

use crate::params::RevenantParams;
use crate::RevenantTaps;

pub fn face(p: &RevenantParams) -> Face {
    let theme = Theme::hammertone();
    let gap = theme.space(4);
    let header = row(vec![
        Title::new("REVENANT").tagline("it learns you · it keeps playing").node(),
        spacer(),
        Presets { width: 184.0 }.node(),
    ])
    .gap(theme.space(3))
    .align(Align::Start);
    let screens = row(vec![
        Display::new(300.0, 210.0, draw_porthole).taps(&RevenantTaps::NEURONS).node(),
        Display::new(300.0, 210.0, draw_meter).taps(&[RevenantTaps::SURPRISE]).node(),
    ])
    .gap(gap)
    .justify(Justify::Center);
    let mind = Plate::titled("MIND").around(
        row(vec![
            Knob::new(p.learn.bind()).node(),
            Knob::new(p.memory.bind()).node(),
            Knob::new(p.chaos.bind()).node(),
            Knob::new(p.horizon.bind()).node(),
        ])
        .gap(gap)
        .align(Align::End),
    );
    let voice = Plate::titled("VOICE").around(
        row(vec![Knob::new(p.dream.bind()).large().node(), Knob::new(p.mix.bind()).large().node()])
            .gap(gap)
            .align(Align::End),
    );
    let controls = row(vec![mind, voice]).gap(gap).align(Align::Stretch);
    let feet = row(vec![
        Footswitch::bypass(p.bypass.bind()).color(hex(0xFF2A1A)).node(),
        column(vec![spacer(), Caption::new("96 neurons · echo-state reservoir").node()]).width(Dim::Fill(1.0)),
        Footswitch::new(p.sleep.bind()).label("SLEEP").color(hex(0x4AA8FF)).node(),
    ])
    .align(Align::Stretch);
    let body = column(vec![header, screens, controls, feet]).gap(theme.space(4)).align(Align::Stretch);
    Face::new(theme, Panel::default().around(body))
}

// ------------------------------------------------------------ the porthole

const N: usize = RevenantTaps::NEURONS.len();
const GLOW_LEVELS: usize = 8;

/// Where neuron `i` floats, in units of the porthole's radius: a
/// golden-angle spiral, jittered — a fixed body plan, the same creature
/// every time.
fn neuron_pos(i: usize) -> (f64, f64) {
    let golden = TAU * (1.0 - 1.0 / 1.618_034);
    let r = 0.8 * ((i as f64 + 0.6) / N as f64).sqrt();
    let a = i as f64 * golden;
    let j = |k: u32| {
        let mut h = (i as u32 + 1).wrapping_mul(0x9E37_79B9) ^ k;
        h ^= h >> 15;
        h = h.wrapping_mul(0x2C1B_3C6D);
        (f64::from(h >> 8) / 16_777_216.0 - 0.5) * 0.11
    };
    (r * a.cos() + j(1), r * a.sin() + j(2))
}

/// The filaments: each neuron to its two nearest neighbours.
fn filaments() -> &'static [(usize, usize)] {
    static F: OnceLock<Vec<(usize, usize)>> = OnceLock::new();
    F.get_or_init(|| {
        let mut out = Vec::new();
        for i in 0..N {
            let (xi, yi) = neuron_pos(i);
            let mut d: Vec<(f64, usize)> = (0..N)
                .filter(|&j| j != i)
                .map(|j| {
                    let (xj, yj) = neuron_pos(j);
                    ((xi - xj).hypot(yi - yj), j)
                })
                .collect();
            d.sort_by(|a, b| a.0.total_cmp(&b.0));
            for &(_, j) in d.iter().take(2) {
                let e = if i < j { (i, j) } else { (j, i) };
                if !out.contains(&e) {
                    out.push(e);
                }
            }
        }
        out
    })
}

/// A neuron's glow at `level` of `GLOW_LEVELS`: a hot soma in a halo.
fn neuron_px(dx: f64, dy: f64, color: Rgb, level: usize, scale: f64) -> Option<(Rgb, f64)> {
    let k = level as f64 / GLOW_LEVELS as f64;
    let r = dx.hypot(dy);
    let soma = 1.5 + 2.0 * k;
    let core = material::cov(r - soma, scale);
    let halo = k * (0.55 * (-(r / (3.0 + 4.0 * k)).powi(2)).exp() + 0.25 * (-(r / (8.0 + 6.0 * k)).powi(2)).exp());
    let hot = (1.0 - r / soma).clamp(0.0, 1.0);
    let c = mix(mul(color, 1.1 + k), [1.8, 2.0, 2.0], hot * k);
    let a = (halo + core * (0.4 + 0.6 * k)).min(1.0);
    (a > 1e-3).then_some((c, a))
}

fn act(f: &Frame<'_>, i: usize) -> f64 {
    let v = f.tap(RevenantTaps::NEURONS[i]);
    if v.is_finite() {
        v.clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

fn draw_porthole(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    let c = pt(r.x + r.w / 2.0, r.y + r.h / 2.0);
    let rad = r.h / 2.0 - 10.0;
    // The fluid: a deep teal well, brighter toward its heart, ringed.
    for k in 0..10 {
        let t = f64::from(k) / 9.0;
        s.solid(Path::circle(c, rad * (1.0 - 0.75 * t)), Color::rgba(0.05, 0.22 + 0.1 * t, 0.26 + 0.1 * t, 0.16 + 0.04 * t));
    }
    s.stroke(Path::circle(c, rad), Stroke::new(Color::rgba(0.44, 0.9, 0.94, 0.28), 1.2));
    s.stroke(Path::circle(c, rad + 5.0), Stroke::new(Color::rgba(0.44, 0.9, 0.94, 0.08), 1.0));
    // Graduations round the rim, like a lab port.
    for k in 0..48 {
        let a = TAU * f64::from(k) / 48.0;
        let (r0, r1) = if k % 4 == 0 { (rad + 3.0, rad + 9.0) } else { (rad + 3.0, rad + 6.0) };
        s.stroke(
            Path::new().move_to(pt(c.x + r0 * a.cos(), c.y + r0 * a.sin())).line_to(pt(c.x + r1 * a.cos(), c.y + r1 * a.sin())),
            Stroke::new(Color::rgba(0.44, 0.9, 0.94, 0.22), 1.0),
        );
    }
    let at = |i: usize| {
        let (x, y) = neuron_pos(i);
        pt(c.x + x * rad, c.y + y * rad)
    };
    // Every filament faintly, lit where both ends are active.
    for &(i, j) in filaments() {
        let a = (act(f, i).abs() * act(f, j).abs()).sqrt();
        s.stroke(
            Path::new().move_to(at(i)).line_to(at(j)),
            Stroke::round(Color::rgba(0.45, 0.95, 1.0, 0.07 + 0.6 * a), 0.6 + 1.6 * a),
        );
    }
    // Each neuron: a dim seed always, glowing with its own activity.
    for i in 0..N {
        let a = act(f, i);
        let p = at(i);
        s.solid(Path::circle(p, 1.4), Color::rgba(0.5, 0.85, 0.9, 0.35));
        let level = (a.abs().powf(0.6) * GLOW_LEVELS as f64).round() as usize;
        if level > 0 {
            let neg = a < 0.0;
            let color: Rgb = if neg { [0.75, 0.4, 1.0] } else { [0.35, 1.0, 1.0] };
            let key = material_key(neg, level);
            f.sprite(s, p, 36.0, key, |sc| material::sprite(36.0, sc, |dx, dy| neuron_px(dx, dy, color, level, sc)));
        }
    }
    let ty = &f.theme.typography;
    let ph = f.theme.palette.phosphor;
    let label = "ORGANISM";
    s.text_in(
        pt(r.x + 12.0 + caption_half_width(label, ty.caption_size), r.y + 18.0),
        label,
        ty.caption_size,
        Color::rgba(ph.r, ph.g, ph.b, 0.7),
        ty.caption,
    );
}

/// Half a tracked-caps caption's width, for setting a centred caption
/// flush to an edge (the author surface has no `TextAlign`).
fn caption_half_width(text: &str, size: f64) -> f64 {
    0.5 * text.chars().count() as f64 * size * 0.78
}

fn material_key(neg: bool, level: usize) -> u64 {
    sprite_key(("revenant-neuron", neg, level))
}

// --------------------------------------------------------------- the meter

/// Needle sweep, y-down degrees (218 = upper left, 322 = upper right).
const SWEEP: (f64, f64) = (218.0, 322.0);

fn draw_meter(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    let ink = Color::rgba(0.12, 0.11, 0.10, 1.0);
    let red = Color::rgba(0.75, 0.13, 0.08, 1.0);
    // The cream card, lamp-lit from above.
    let card = Rect { x: r.x + 8.0, y: r.y + 8.0, w: r.w - 16.0, h: r.h - 16.0 };
    vertical_gradient(
        s,
        card,
        4.0,
        (card.y, card.y + card.h),
        &[(0.0, Color::rgba(0.95, 0.92, 0.84, 1.0)), (1.0, Color::rgba(0.86, 0.82, 0.72, 1.0))],
    );
    let pivot = pt(card.x + card.w / 2.0, card.y + card.h - 22.0);
    // The point `r` from the pivot at y-down angle `deg`.
    let polar = |r: f64, deg: f64| {
        let a = deg.to_radians();
        pt(pivot.x + r * a.cos(), pivot.y + r * a.sin())
    };
    let arc_r = card.h - 64.0;
    let needle = arc_r + 14.0;
    let deg = |t: f64| SWEEP.0 + (SWEEP.1 - SWEEP.0) * t;
    // Scale: the arc, ticks, the red zone past 75%.
    let n = 48;
    let arc: Vec<_> = (0..=n).map(|i| polar(arc_r, deg(f64::from(i) / f64::from(n)))).collect();
    s.stroke(Path::polyline(&arc), Stroke::new(ink, 1.3));
    let zone: Vec<_> = (0..=12).map(|i| polar(arc_r + 4.0, deg(0.75 + 0.25 * f64::from(i) / 12.0))).collect();
    s.stroke(Path::polyline(&zone), Stroke::new(red, 4.0));
    for k in 0..=20 {
        let d = deg(f64::from(k) / 20.0);
        let len = if k % 5 == 0 { 10.0 } else { 5.0 };
        s.stroke(Path::new().move_to(polar(arc_r, d)).line_to(polar(arc_r + len, d)), Stroke::new(ink, if k % 5 == 0 { 1.4 } else { 0.9 }));
    }
    let ty = &f.theme.typography;
    // "calm" ends at the sweep's start, "lost" begins at its end.
    let size = ty.caption_size + 1.0;
    let (calm, lost) = (polar(arc_r + 18.0, deg(0.0)), polar(arc_r + 18.0, deg(1.0)));
    s.text_in(pt(calm.x - caption_half_width("calm", size), calm.y), "calm", size, ink, ty.caption);
    s.text_in(pt(lost.x + caption_half_width("lost", size), lost.y), "lost", size, red, ty.caption);
    s.text_in(pt(pivot.x, pivot.y - arc_r * 0.42), "SURPRISE", ty.legend_size + 1.5, ink, ty.legend);

    // The needle (its shadow first), a red tip, the pivot boss.
    let v = Some(f.tap(RevenantTaps::SURPRISE)).filter(|v| v.is_finite()).unwrap_or(0.0).clamp(0.0, 1.0);
    let d = deg(v);
    let tip = polar(needle, d);
    s.stroke(Path::new().move_to(pt(pivot.x + 2.5, pivot.y + 3.5)).line_to(pt(tip.x + 2.5, tip.y + 3.5)), Stroke::round(Color::rgba(0.0, 0.0, 0.0, 0.22), 2.0));
    s.stroke(Path::new().move_to(pivot).line_to(tip), Stroke::round(Color::rgba(0.08, 0.07, 0.06, 1.0), 1.6));
    s.stroke(Path::new().move_to(polar(needle - 16.0, d)).line_to(tip), Stroke::round(red, 1.6));
    s.solid(Path::circle(pivot, 7.0), Color::rgba(0.18, 0.18, 0.19, 1.0));
    s.solid(Path::circle(pt(pivot.x - 1.5, pivot.y - 1.5), 3.0), Color::rgba(0.55, 0.55, 0.57, 1.0));
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
