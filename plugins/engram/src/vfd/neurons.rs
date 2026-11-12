//! The NEURONS window: the room's sixteen delay lines on their field
//! (radius from the centre = a line's length, closeness = coupling),
//! glowing with their energy and tethered to where plasticity has
//! pulled them. Grab a neuron and drag it to place it.
//!
//! The nodes move, so the window places its grab targets from live
//! values (`Part::hit`): a press near a neuron's drawn (or live)
//! position becomes a pad on that neuron's x/y over the field. The
//! field draws y DOWN (as the pedal face did), so its pad area is the
//! field flipped (bottom edge, negative height): the runtime's pad maps
//! y bottom→top over its area, which over a flipped area is top→bottom
//! — the value a press sets is the one drawn under the pointer.

use stave_core::face::part::Bake;
use stave_core::face::prelude::*;
use stave_core::face::{leaf, Bind, Gesture, Part};

use super::{glow, Feed, Pt, TextAlign, LINES, SCREEN_H};
use crate::FIELD_ASPECT;

/// The window's screen width (its height is [`SCREEN_H`]).
pub const W: f64 = 340.0;

const LIP: f64 = 5.0;
const RADIUS: f64 = 7.0;
const SIDE: f64 = 10.0;
const FIELD_TOP: f64 = 32.0;
/// A press this close (pt) to a neuron grabs it.
pub const GRAB: f64 = 14.0;
/// Coupling falls off over this distance (field heights).
const REACH: f64 = 0.22;

/// Every neuron's (x, y) parameters, in order.
pub type Nodes = [(Bind, Bind); LINES];

/// The window.
pub fn network(nodes: Nodes) -> FaceNode {
    leaf(Network { nodes }, W + 2.0 * LIP, SCREEN_H + 2.0 * LIP)
}

struct Network {
    nodes: Nodes,
}

/// The screen inside the window's bezel at `r`.
pub fn screen(r: Rect) -> Rect {
    Rect { x: r.x + LIP, y: r.y + LIP, w: r.w - 2.0 * LIP, h: r.h - 2.0 * LIP }
}

/// The field inside screen `sc`: its width is FIELD_ASPECT heights
/// (geometry is measured as the face draws it).
pub fn field(sc: Rect) -> Rect {
    let w = sc.w - 2.0 * SIDE;
    Rect { x: sc.x + SIDE, y: sc.y + FIELD_TOP, w, h: w / FIELD_ASPECT }
}

/// A field point (x right, y down) on screen.
pub fn at(fr: Rect, (u, v): (f64, f64)) -> Pt {
    pt(fr.x + u * fr.w, fr.y + v * fr.h)
}

/// The pad area for field `fr`: the field flipped, so the runtime's
/// y-up pad lands on the y-down drawing (see the module doc).
pub fn pad_area(fr: Rect) -> Rect {
    Rect { x: fr.x, y: fr.y + fr.h, w: fr.w, h: -fr.h }
}

fn ellipse(c: Pt, rx: f64, ry: f64) -> Path {
    let pts: Vec<Pt> = (0..=72)
        .map(|k| {
            let a = core::f64::consts::TAU * f64::from(k) / 72.0;
            pt(c.x + rx * a.cos(), c.y + ry * a.sin())
        })
        .collect();
    Path::polyline(&pts)
}

impl Network {
    /// Neuron `i`'s drawn position (its params) and live one (the
    /// feed; the drawing until the engine has spoken).
    fn node_at(&self, f: &Frame<'_>, feed: &Feed, i: usize) -> ((f64, f64), (f64, f64)) {
        let (bx, by) = self.nodes[i];
        let drawn = (f.value(bx), f.value(by));
        let x = feed.pos.get(2 * i).copied().unwrap_or(0.0);
        let y = feed.pos.get(2 * i + 1).copied().unwrap_or(0.0);
        (drawn, if x != 0.0 || y != 0.0 { (x, y) } else { drawn })
    }
}

impl Part for Network {
    fn bake(&self, r: Rect, cx: &mut Bake<'_>) {
        bake_window(cx, screen(r), RADIUS, LIP);
        let th = cx.theme;
        let sc = screen(r);
        let fr = field(sc);
        // The field's guides, unlit: rings of equal line length, and a
        // crosshair — phosphor that is merely there.
        let c = at(fr, (0.5, 0.5));
        for ring in [0.33, 0.66, 1.0] {
            cx.scene.stroke(ellipse(c, ring * fr.w / 2.0, ring * fr.h / 2.0), Stroke::new(glow(th, 0.07), 0.8));
        }
        let cross = Stroke::new(glow(th, 0.05), 0.8);
        cx.scene.stroke(Path::new().move_to(pt(fr.x, c.y)).line_to(pt(fr.x + fr.w, c.y)), cross);
        cx.scene.stroke(Path::new().move_to(pt(c.x, fr.y)).line_to(pt(c.x, fr.y + fr.h)), cross);
        let ty = &th.type_;
        let font = ty.caption.align(TextAlign::Left);
        cx.scene.text_in(pt(sc.x + 12.0, sc.y + 17.0), "NEURONS", 7.0, glow(th, 0.6), font);
        cx.scene.text_in(
            pt(sc.x + sc.w - 12.0, sc.y + 17.0),
            "DRAG A NEURON TO PLACE IT",
            7.0,
            glow(th, 0.6),
            font.align(TextAlign::Right),
        );
    }

    fn paint(&self, r: Rect, f: &Frame<'_>, s: &mut Scene) {
        let th = f.theme;
        let sc = screen(r);
        let fr = field(sc);
        let feed = Feed::read(f);
        let energy = |i: usize| feed.energy.get(i).copied().unwrap_or(0.0).clamp(0.0, 1.0);
        let nodes: Vec<_> = (0..LINES).map(|i| self.node_at(f, &feed, i)).collect();

        // Edges by closeness, lit by the energy flowing along them.
        for i in 0..LINES {
            for j in i + 1..LINES {
                let (a, b) = (nodes[i].1, nodes[j].1);
                let d = ((a.0 - b.0) * FIELD_ASPECT).hypot(a.1 - b.1) / REACH;
                let c = (-d * d).exp();
                if c > 0.06 {
                    let lit = (energy(i) * energy(j)).sqrt();
                    s.stroke(
                        Path::new().move_to(at(fr, a)).line_to(at(fr, b)),
                        Stroke::round(glow(th, 0.10 + 0.25 * c + 0.45 * c * lit), 0.6 + 1.4 * c),
                    );
                }
            }
        }
        // Tethers from the drawing to where plasticity pulled each one.
        for (drawn, live) in &nodes {
            let (pd, pl) = (at(fr, *drawn), at(fr, *live));
            if (pd.x - pl.x).hypot(pd.y - pl.y) > 2.0 {
                s.stroke(Path::new().move_to(pd).line_to(pl), Stroke::round(glow(th, 0.3), 0.6));
                s.stroke(Path::circle(pd, 3.5), Stroke::new(glow(th, 0.35), 0.8));
            }
        }
        for (i, (_, live)) in nodes.iter().enumerate() {
            let p = at(fr, *live);
            let e = energy(i);
            s.solid(Path::circle(p, 6.0 + 8.0 * e), glow(th, 0.06 + 0.22 * e));
            s.solid(Path::circle(p, 3.0 + 1.5 * e), glow(th, 0.55 + 0.45 * e));
            s.solid(Path::circle(p, 1.3), Color::rgba(0.9, 1.0, 0.97, 0.6 + 0.4 * e));
        }
        glass_over(f, s, sc, RADIUS);
    }

    fn hit(&self, r: Rect, f: &Frame<'_>, p: Pt) -> Option<Gesture> {
        let fr = field(screen(r));
        let feed = Feed::read(f);
        let mut best: Option<(usize, f64)> = None;
        for i in 0..LINES {
            let (drawn, live) = self.node_at(f, &feed, i);
            for spot in [drawn, live] {
                let q = at(fr, spot);
                let d = (q.x - p.x).hypot(q.y - p.y);
                if d <= GRAB && best.is_none_or(|(_, b)| d < b) {
                    best = Some((i, d));
                }
            }
        }
        let (i, _) = best?;
        let (x, y) = self.nodes[i];
        Some(Gesture::Pad { x, y, area: pad_area(fr) })
    }

    fn probes(&self) -> Vec<&'static str> {
        crate::FACE_TAPS.to_vec()
    }

    fn reads(&self) -> Vec<Bind> {
        self.nodes.iter().flat_map(|&(x, y)| [x, y]).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SC: Rect = Rect { x: 0.0, y: 0.0, w: W, h: SCREEN_H };

    #[test]
    fn the_field_keeps_its_aspect_and_fits_the_screen() {
        let fr = field(SC);
        assert!((fr.w / fr.h - FIELD_ASPECT).abs() < 1e-9);
        assert!(fr.y > SC.y + 18.0, "field under the labels");
        assert!(fr.y + fr.h <= SC.y + SC.h - 4.0, "field inside the screen");
    }

    #[test]
    fn a_field_point_round_trips_through_the_flipped_pad() {
        // The runtime's pad maps a pointer to (x, 1 − y) over its area;
        // over the flipped area that is the y-down drawing's value.
        let fr = field(SC);
        let a = pad_area(fr);
        let p = at(fr, (0.25, 0.8));
        let (nx, ny) = ((p.x - a.x) / a.w, 1.0 - (p.y - a.y) / a.h);
        assert!((nx - 0.25).abs() < 1e-9 && (ny - 0.8).abs() < 1e-9);
    }
}
