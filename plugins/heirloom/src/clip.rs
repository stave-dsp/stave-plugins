//! Clipping to a screen rect, for display pictures that run off the
//! glass (the Scene IR has no clip op). Polygons clip exactly
//! (Sutherland–Hodgman); polylines split into the runs inside.

use stave_ui::layout::Rect;
use stave_ui::scene::{pt, Path, Pt};

/// One clip edge: is a point inside it, and where a segment crosses it.
type Edge = (fn(Pt, Rect) -> bool, fn(Pt, Pt, Rect) -> Pt);

/// `pts` (a closed convex-or-not polygon) clipped to `r`, as a fill path.
pub fn polygon(pts: &[Pt], r: Rect) -> Option<Path> {
    let edges: [Edge; 4] = [
        (|p, r| p.x >= r.x, |a, b, r| lerp_x(a, b, r.x)),
        (|p, r| p.x <= r.x + r.w, |a, b, r| lerp_x(a, b, r.x + r.w)),
        (|p, r| p.y >= r.y, |a, b, r| lerp_y(a, b, r.y)),
        (|p, r| p.y <= r.y + r.h, |a, b, r| lerp_y(a, b, r.y + r.h)),
    ];
    let mut out: Vec<Pt> = pts.to_vec();
    for (inside, cut) in edges {
        if out.is_empty() {
            return None;
        }
        let inp = core::mem::take(&mut out);
        let mut prev = *inp.last().unwrap();
        for &p in &inp {
            match (inside(p, r), inside(prev, r)) {
                (true, true) => out.push(p),
                (true, false) => {
                    out.push(cut(prev, p, r));
                    out.push(p);
                }
                (false, true) => out.push(cut(prev, p, r)),
                (false, false) => {}
            }
            prev = p;
        }
    }
    (out.len() >= 3).then(|| Path::polyline(&out).close())
}

/// `pts` (an open polyline) clipped to `r`: one path per run inside.
pub fn polyline(pts: &[Pt], r: Rect) -> Vec<Path> {
    let mut runs = Vec::new();
    let mut cur: Vec<Pt> = Vec::new();
    for w in pts.windows(2) {
        match segment(w[0], w[1], r) {
            Some((a, b)) => {
                if cur.last().is_none_or(|l| (l.x - a.x).abs() + (l.y - a.y).abs() > 1e-6) {
                    if cur.len() >= 2 {
                        runs.push(Path::polyline(&cur));
                    }
                    cur.clear();
                    cur.push(a);
                }
                cur.push(b);
            }
            None => {
                if cur.len() >= 2 {
                    runs.push(Path::polyline(&cur));
                }
                cur.clear();
            }
        }
    }
    if cur.len() >= 2 {
        runs.push(Path::polyline(&cur));
    }
    runs
}

/// A circle of radius `rad` about `c` as a closed polyline.
pub fn circle(c: Pt, rad: f64, n: usize) -> Vec<Pt> {
    (0..=n)
        .map(|i| {
            let a = core::f64::consts::TAU * i as f64 / n as f64;
            pt(c.x + rad * a.cos(), c.y + rad * a.sin())
        })
        .collect()
}

fn lerp_x(a: Pt, b: Pt, x: f64) -> Pt {
    let t = (x - a.x) / (b.x - a.x);
    pt(x, a.y + (b.y - a.y) * t)
}

fn lerp_y(a: Pt, b: Pt, y: f64) -> Pt {
    let t = (y - a.y) / (b.y - a.y);
    pt(a.x + (b.x - a.x) * t, y)
}

/// Liang–Barsky: the part of segment a→b inside `r`.
fn segment(a: Pt, b: Pt, r: Rect) -> Option<(Pt, Pt)> {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let (mut t0, mut t1) = (0.0f64, 1.0f64);
    for (p, q) in [(-dx, a.x - r.x), (dx, r.x + r.w - a.x), (-dy, a.y - r.y), (dy, r.y + r.h - a.y)] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                t0 = t0.max(t);
            } else {
                t1 = t1.min(t);
            }
        }
    }
    (t0 <= t1).then(|| (pt(a.x + dx * t0, a.y + dy * t0), pt(a.x + dx * t1, a.y + dy * t1)))
}
