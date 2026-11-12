//! Clipping to a screen rect, for display pictures that run off the
//! glass (the Scene IR has no clip op). Polygons clip exactly
//! (Sutherland–Hodgman); polylines split into the runs inside.

use stave::prelude::*;

/// A point, `(x, y)`. (The face API names no point type, so the
/// clipper keeps its own and makes `Path`s at the edge — see `path`.)
pub type P = (f64, f64);

/// A polyline through `pts`.
fn path(pts: &[P]) -> Path {
    let pts: Vec<_> = pts.iter().map(|&(x, y)| pt(x, y)).collect();
    Path::polyline(&pts)
}

/// One clip edge: is a point inside it, and where a segment crosses it.
type Edge = (fn(P, Rect) -> bool, fn(P, P, Rect) -> P);

/// `pts` (a closed convex-or-not polygon) clipped to `r`, as a fill path.
pub fn polygon(pts: &[P], r: Rect) -> Option<Path> {
    let edges: [Edge; 4] = [
        (|p, r| p.0 >= r.x, |a, b, r| lerp_x(a, b, r.x)),
        (|p, r| p.0 <= r.x + r.w, |a, b, r| lerp_x(a, b, r.x + r.w)),
        (|p, r| p.1 >= r.y, |a, b, r| lerp_y(a, b, r.y)),
        (|p, r| p.1 <= r.y + r.h, |a, b, r| lerp_y(a, b, r.y + r.h)),
    ];
    let mut out: Vec<P> = pts.to_vec();
    for (inside, cut) in edges {
        if out.is_empty() {
            return None;
        }
        let inp = std::mem::take(&mut out);
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
    (out.len() >= 3).then(|| path(&out).close())
}

/// `pts` (an open polyline) clipped to `r`: one path per run inside.
pub fn polyline(pts: &[P], r: Rect) -> Vec<Path> {
    let mut runs = Vec::new();
    let mut cur: Vec<P> = Vec::new();
    for w in pts.windows(2) {
        match segment(w[0], w[1], r) {
            Some((a, b)) => {
                if cur.last().is_none_or(|l| (l.0 - a.0).abs() + (l.1 - a.1).abs() > 1e-6) {
                    if cur.len() >= 2 {
                        runs.push(path(&cur));
                    }
                    cur.clear();
                    cur.push(a);
                }
                cur.push(b);
            }
            None => {
                if cur.len() >= 2 {
                    runs.push(path(&cur));
                }
                cur.clear();
            }
        }
    }
    if cur.len() >= 2 {
        runs.push(path(&cur));
    }
    runs
}

/// A circle of radius `rad` about `c` as a closed polyline.
pub fn circle(c: P, rad: f64, n: usize) -> Vec<P> {
    (0..=n)
        .map(|i| {
            let a = std::f64::consts::TAU * i as f64 / n as f64;
            (c.0 + rad * a.cos(), c.1 + rad * a.sin())
        })
        .collect()
}

fn lerp_x(a: P, b: P, x: f64) -> P {
    let t = (x - a.0) / (b.0 - a.0);
    (x, a.1 + (b.1 - a.1) * t)
}

fn lerp_y(a: P, b: P, y: f64) -> P {
    let t = (y - a.1) / (b.1 - a.1);
    (a.0 + (b.0 - a.0) * t, y)
}

/// Liang–Barsky: the part of segment a→b inside `r`.
fn segment(a: P, b: P, r: Rect) -> Option<(P, P)> {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let (mut t0, mut t1) = (0.0f64, 1.0f64);
    for (p, q) in [(-dx, a.0 - r.x), (dx, r.x + r.w - a.0), (-dy, a.1 - r.y), (dy, r.y + r.h - a.1)] {
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
    (t0 <= t1).then_some(((a.0 + dx * t0, a.1 + dy * t0), (a.0 + dx * t1, a.1 + dy * t1)))
}
