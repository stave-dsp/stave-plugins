//! The harp window's picture: twelve strings strung between brass
//! bridges over black felt. Each string vibrates and glows with its own
//! ring (the `ring_*` taps), in time with the harp's clock (`clock`) — the display IS the instrument.

use core::f64::consts::{PI, TAU};

use stave::prelude::*;

use crate::AeolianTaps;

const NOTES: [&str; 12] = ["C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B"];

/// Clear of the bezel either side of the outer strings.
const SIDE: f64 = 36.0;
/// The bridges, in from the top and up from the bottom (room for the
/// note names under the lower one).
const BRIDGE_TOP: f64 = 22.0;
const BRIDGE_BOT: f64 = 34.0;

const BRASS_HI: Color = Color::rgba(0.98, 0.84, 0.52, 1.0);
const BRASS: Color = Color::rgba(0.78, 0.58, 0.28, 1.0);
const BRASS_LO: Color = Color::rgba(0.36, 0.24, 0.10, 1.0);

/// The strings' x positions in screen `r`.
pub fn string_x(r: Rect, c: usize) -> f64 {
    r.x + SIDE + (r.w - 2.0 * SIDE) * c as f64 / 11.0
}

fn bridges(r: Rect) -> (f64, f64) {
    (r.y + BRIDGE_TOP, r.y + r.h - BRIDGE_BOT)
}

/// The window: felt, bridges and pins, the strings, the note names.
pub fn draw(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    felt(r, s);
    let (top, bot) = bridges(r);
    let t = finite(f.tap(AeolianTaps::CLOCK));
    let pitch = (r.w - 2.0 * SIDE) / 11.0;
    for (c, tap) in AeolianTaps::RINGS.into_iter().enumerate() {
        let level = finite(f.tap(tap)).clamp(0.0, 1.0);
        string(s, string_x(r, c), top + 4.0, bot - 4.0, c, level, t, pitch);
    }
    for &y in &[top, bot] {
        bridge(r, s, y);
    }
    let ty = &f.theme.typography;
    for (c, name) in NOTES.iter().enumerate() {
        let sharp = name.chars().count() > 1;
        let (size, ink) = if sharp { (ty.caption_size, f.theme.palette.ink_soft) } else { (ty.legend_size, f.theme.palette.ink) };
        s.text_in(pt(string_x(r, c), r.y + r.h - 11.0), *name, size, ink, ty.legend);
    }
}

fn finite(v: f64) -> f64 {
    if v.is_finite() {
        v
    } else {
        0.0
    }
}

/// Black felt, lit from above and falling into the bezel's shadow.
fn felt(r: Rect, s: &mut Scene) {
    vertical_gradient(
        s,
        r,
        7.0,
        (r.y, r.y + r.h),
        &[
            (0.0, Color::rgba(0.02, 0.03, 0.03, 1.0)),
            (0.18, Color::rgba(0.075, 0.08, 0.08, 1.0)),
            (0.75, Color::rgba(0.055, 0.06, 0.06, 1.0)),
            (1.0, Color::rgba(0.025, 0.03, 0.03, 1.0)),
        ],
    );
}

/// A brass bar across the window at `y`, with a domed pin per string.
fn bridge(r: Rect, s: &mut Scene, y: f64) {
    let (x0, x1) = (r.x + 14.0, r.x + r.w - 14.0);
    s.solid(Path::rect(pt(x0, y + 3.0), x1 - x0, 5.0), Color::rgba(0.0, 0.0, 0.0, 0.45));
    vertical_gradient(
        s,
        Rect { x: x0, y: y - 3.5, w: x1 - x0, h: 7.0 },
        3.0,
        (y - 3.5, y + 3.5),
        &[(0.0, BRASS_LO), (0.3, BRASS_HI), (0.6, BRASS), (1.0, BRASS_LO)],
    );
    for c in 0..12 {
        let p = pt(string_x(r, c), y);
        s.solid(Path::circle(pt(p.x + 0.6, p.y + 1.0), 3.0), Color::rgba(0.0, 0.0, 0.0, 0.5));
        s.solid(Path::circle(p, 2.8), BRASS);
        s.solid(Path::circle(pt(p.x - 0.8, p.y - 0.9), 1.1), BRASS_HI);
    }
}

/// One string at ring `level` (0..1), clock `t` seconds: a standing
/// wave between the bridges, the blur a fast string leaves in the eye,
/// and a bloom as it sings. Wound bronze in the bass, steel in the treble.
#[allow(clippy::too_many_arguments)]
fn string(s: &mut Scene, x: f64, y0: f64, y1: f64, c: usize, level: f64, t: f64, pitch: f64) {
    const SEGS: usize = 28;
    let k = c as f64 / 11.0;
    let base = Color::rgba(0.80 - 0.05 * k, 0.58 + 0.17 * k, 0.34 + 0.40 * k, 1.0);
    let width = 2.0 - 0.8 * k;
    let amp = 0.2 * pitch * level.powf(1.3);
    let phase = (t * (6.0 + 0.85 * c as f64) * TAU).sin();
    let pts = |off: f64, a: f64| -> Vec<_> {
        (0..=SEGS)
            .map(|i| {
                let u = i as f64 / SEGS as f64;
                pt(x + off + a * (PI * u).sin(), y0 + (y1 - y0) * u)
            })
            .collect()
    };
    // Shadow on the felt.
    s.stroke(Path::polyline(&pts(2.4, amp * phase * 0.8)), Stroke::round(Color::rgba(0.0, 0.0, 0.0, 0.6), width + 1.2));
    if level > 0.02 {
        let mut env = Path::polyline(&pts(0.0, amp));
        for p in pts(0.0, -amp).into_iter().rev() {
            env = env.line_to(p);
        }
        s.solid(env.close(), Color::rgba(1.0, 0.86, 0.6, 0.10 + 0.22 * level));
        s.stroke(
            Path::polyline(&pts(0.0, amp * phase)),
            Stroke::round(Color::rgba(1.0, 0.8, 0.5, 0.25 * level), width + 6.0),
        );
    }
    let lit = |v: f64, w: f64| v + (w - v) * 0.6 * level;
    let col = Color::rgba(lit(base.r, 1.0), lit(base.g, 0.95), lit(base.b, 0.85), 1.0);
    s.stroke(Path::polyline(&pts(0.0, amp * phase)), Stroke::round(col, width));
    s.stroke(
        Path::polyline(&pts(-0.35 * width, amp * phase)),
        Stroke::round(Color::rgba(1.0, 1.0, 1.0, 0.45 + 0.3 * level), 0.55),
    );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_sit_inside_the_window_evenly() {
        let r = Rect { x: 10.0, y: 10.0, w: 440.0, h: 190.0 };
        assert!(string_x(r, 0) > r.x + 20.0 && string_x(r, 11) < r.x + r.w - 20.0);
        let p0 = string_x(r, 1) - string_x(r, 0);
        let p1 = string_x(r, 11) - string_x(r, 10);
        assert!((p0 - p1).abs() < 1e-9);
    }
}
