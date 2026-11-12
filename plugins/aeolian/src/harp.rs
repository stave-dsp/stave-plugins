//! The harp window's picture: twelve strings strung between brass
//! bridges over black felt. Each string vibrates and glows with its own
//! ring (the `ring <note>` taps), in time with the harp's clock (`harp
//! clock`) — the display IS the instrument.

use core::f64::consts::{PI, TAU};

use stave_core::face::prelude::*;
use stave_core::ui::scene::{Brush, Pt};

use crate::{CLOCK_TAP, LED_TAPS};

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
    let t = finite(f.probe(CLOCK_TAP));
    let pitch = (r.w - 2.0 * SIDE) / 11.0;
    for (c, tap) in LED_TAPS.iter().enumerate() {
        let level = finite(f.probe(tap)).clamp(0.0, 1.0);
        string(s, string_x(r, c), top + 4.0, bot - 4.0, c, level, t, pitch);
    }
    for &y in &[top, bot] {
        bridge(r, s, y);
    }
    let ty = &f.theme.type_;
    for (c, name) in NOTES.iter().enumerate() {
        let sharp = name.chars().count() > 1;
        let (size, ink) = if sharp { (ty.caption_size, f.theme.ink_soft) } else { (ty.legend_size, f.theme.ink) };
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
    let path = rrect_path(r, 7.0);
    s.fill(
        path,
        Brush::Linear {
            from: pt(r.x, r.y),
            to: pt(r.x, r.y + r.h),
            stops: vec![
                (0.0, Color::rgba(0.02, 0.03, 0.03, 1.0)),
                (0.18, Color::rgba(0.075, 0.08, 0.08, 1.0)),
                (0.75, Color::rgba(0.055, 0.06, 0.06, 1.0)),
                (1.0, Color::rgba(0.025, 0.03, 0.03, 1.0)),
            ],
        },
    );
}

/// A brass bar across the window at `y`, with a domed pin per string.
fn bridge(r: Rect, s: &mut Scene, y: f64) {
    let (x0, x1) = (r.x + 14.0, r.x + r.w - 14.0);
    s.solid(Path::rect(pt(x0, y + 3.0), x1 - x0, 5.0), Color::rgba(0.0, 0.0, 0.0, 0.45));
    s.fill(
        rrect_path(Rect { x: x0, y: y - 3.5, w: x1 - x0, h: 7.0 }, 3.0),
        Brush::Linear {
            from: pt(0.0, y - 3.5),
            to: pt(0.0, y + 3.5),
            stops: vec![(0.0, BRASS_LO), (0.3, BRASS_HI), (0.6, BRASS), (1.0, BRASS_LO)],
        },
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
    let pts = |off: f64, a: f64| -> Vec<Pt> {
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
