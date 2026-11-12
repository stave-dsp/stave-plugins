//! The SPACE window: the MAP of the ≈700 real reverbs the room learned,
//! each a faint unlit dot, families labelled. Click or drag to put the
//! cursor (SPACE X / Y) anywhere; the live room (the bright blob) glides
//! after it, wanders with DRIFT, and is pulled (an amber line) toward
//! where the recalled memories were heard — each memory a dot at its
//! room, blazing with the attention it gets. Underneath: the nearest
//! real space and the decay.

use std::sync::OnceLock;

use stave::face::part::Bake;
use stave::face::{leaf, Area, Gesture, Grab, Part};
use stave::prelude::*;

use super::{amber, glow, label, print, Anchor, Feed, SCREEN_H, SLOTS};
use crate::engine::space_model::{FAMILIES, MAP_FAMILY, MAP_XY, NAMES, SPACES};
use crate::EngramTaps;

/// The window's screen width (its height is [`SCREEN_H`]).
pub const W: f64 = 228.0;

const LIP: f64 = 5.0;
const RADIUS: f64 = 7.0;
const MAP_TOP: f64 = 26.0;
const MAP_SIZE: f64 = 172.0;

/// The map window, bound to the cursor's x/y.
pub fn map(x: Bind, y: Bind) -> FaceNode {
    leaf(SpaceMap { x, y }, W + 2.0 * LIP, SCREEN_H + 2.0 * LIP)
}

struct SpaceMap {
    x: Bind,
    y: Bind,
}

fn screen(r: Rect) -> Rect {
    Rect { x: r.x + LIP, y: r.y + LIP, w: r.w - 2.0 * LIP, h: r.h - 2.0 * LIP }
}

/// The square map inside screen `sc`.
pub fn area(sc: Rect) -> Rect {
    Rect { x: sc.x + (sc.w - MAP_SIZE) / 2.0, y: sc.y + MAP_TOP, w: MAP_SIZE, h: MAP_SIZE }
}

/// A map point (x right, y up) on screen.
pub fn at(m: Rect, (u, v): (f64, f64)) -> (f64, f64) {
    (m.x + u * m.w, m.y + (1.0 - v) * m.h)
}

fn room(i: usize) -> (f64, f64) {
    (f64::from(MAP_XY[2 * i]), f64::from(MAP_XY[2 * i + 1]))
}

/// The families' label spots: each family's median position, skipping
/// any too close to one already placed (computed once).
fn family_labels() -> &'static [(f64, f64, &'static str)] {
    static LABELS: OnceLock<Vec<(f64, f64, &'static str)>> = OnceLock::new();
    LABELS.get_or_init(|| {
        let mut fams: Vec<(usize, Vec<(f64, f64)>)> = (0..FAMILIES.len())
            .map(|k| (k, (0..SPACES).filter(|&i| MAP_FAMILY[i] as usize == k).map(room).collect()))
            .collect();
        fams.sort_by_key(|(_, pts)| core::cmp::Reverse(pts.len()));
        let median = |mut v: Vec<f64>| {
            v.sort_by(f64::total_cmp);
            v[v.len() / 2]
        };
        let mut out: Vec<(f64, f64, &'static str)> = Vec::new();
        for (k, pts) in fams {
            if pts.len() < 12 {
                continue;
            }
            let (x, y) = (median(pts.iter().map(|p| p.0).collect()), median(pts.iter().map(|p| p.1).collect()));
            if out.iter().all(|&(ox, oy, _)| (ox - x).hypot((oy - y) * 2.5) > 0.14) {
                out.push((x, y, FAMILIES[k]));
            }
        }
        out
    })
}

impl Part for SpaceMap {
    fn bake(&self, r: Rect, cx: &mut Bake<'_>) {
        let sc = screen(r);
        bake_window(cx, sc, RADIUS, LIP);
        let th = cx.theme;
        let m = area(sc);
        let at = |uv| {
            let (x, y) = at(m, uv);
            pt(x, y)
        };
        // A faint frame round the map, and every learned room, unlit.
        let frame = Rect { x: m.x - 4.0, y: m.y - 4.0, w: m.w + 8.0, h: m.h + 8.0 };
        cx.scene.stroke(rrect_path(frame, 3.0), Stroke::new(glow(th, 0.1), 0.7));
        let dot = glow(th, 0.16);
        for i in 0..SPACES {
            cx.scene.solid(Path::circle(at(room(i)), 0.9), dot);
        }
        for &(x, y, name) in family_labels() {
            print(th, cx.scene, self::at(m, (x, y)), name, 5.5, glow(th, 0.38), Anchor::Middle);
        }
        print(th, cx.scene, (sc.x + sc.w / 2.0, sc.y + 17.0), "SPACE  ·  CLICK A ROOM", 7.0, glow(th, 0.6), Anchor::Middle);
    }

    fn paint(&self, r: Rect, f: &Frame<'_>, s: &mut Scene) {
        let th = f.theme;
        let sc = screen(r);
        let m = area(sc);
        let at = |uv| {
            let (x, y) = at(m, uv);
            pt(x, y)
        };
        let feed = Feed::read(f);
        // Memories, at the rooms they were heard in.
        for k in 0..SLOTS {
            if !feed.held(k) {
                continue;
            }
            let u = feed.heard_at.get(2 * k).copied().unwrap_or(0.0);
            let v = feed.heard_at.get(2 * k + 1).copied().unwrap_or(0.0);
            let a = feed.attention(k);
            let p = at((u, v));
            if a > 0.05 {
                s.solid(Path::circle(p, 3.0 + 4.0 * a), feed.ink(th, k, 0.2 * a));
            }
            s.solid(Path::rect(pt(p.x - 1.2, p.y - 1.2), 2.4, 2.4), feed.ink(th, k, 0.3 + 0.7 * a));
        }
        // The cursor (where you put it) and the live room (where it is).
        let cur = at((f.value(self.x), f.value(self.y)));
        let spoken = feed.live != (0.0, 0.0);
        let live = if spoken { at(feed.live) } else { cur };
        if (cur.x - live.x).hypot(cur.y - live.y) > 3.0 {
            s.stroke(Path::new().move_to(cur).line_to(live), Stroke::round(glow(th, 0.25), 0.6));
        }
        let hot = if f.focus.active || f.focus.hover { 1.0 } else { 0.75 };
        s.stroke(Path::circle(cur, 6.5), Stroke::new(glow(th, hot), 0.9));
        for (dx, dy) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
            s.stroke(
                Path::new().move_to(pt(cur.x + 8.5 * dx, cur.y + 8.5 * dy)).line_to(pt(cur.x + 12.0 * dx, cur.y + 12.0 * dy)),
                Stroke::round(glow(th, 0.8 * hot), 0.9),
            );
        }
        if feed.pull > 0.02 {
            let to = at(feed.pull_at);
            s.stroke(Path::new().move_to(live).line_to(to), Stroke::round(amber(0.25 + 0.6 * feed.pull), 0.6 + 1.2 * feed.pull));
            s.stroke(Path::circle(to, 3.0 + 3.0 * feed.pull), Stroke::new(amber(0.4 + 0.6 * feed.pull), 0.9));
        }
        s.solid(Path::circle(live, 9.0), glow(th, 0.16));
        s.solid(Path::circle(live, 3.4), glow(th, 0.95));
        s.solid(Path::circle(live, 1.4), Color::rgba(0.9, 1.0, 0.97, 1.0));
        // The readout: the nearest real space and the decay, once the
        // engine has spoken.
        let line = if spoken {
            let near = ((feed.near * (SPACES - 1) as f64).round() as usize).min(SPACES - 1);
            format!("{}  ·  T60 {:.1} s", NAMES[near], feed.t60)
        } else {
            "—".into()
        };
        label(f, s, (sc.x + sc.w / 2.0, m.y + m.h + 18.0), line, 0.7, Anchor::Middle);
        glass_over(f, s, sc, RADIUS);
    }

    fn grabs(&self, r: Rect, _f: &Frame<'_>, out: &mut Vec<Grab>) {
        let m = area(screen(r));
        out.push(Grab { area: Area::Rect(m), gesture: Gesture::Pad { x: self.x, y: self.y, area: m } });
    }

    fn reads(&self) -> Reads {
        Reads::params(&[self.x, self.y]).and_taps(EngramTaps::ALL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SC: Rect = Rect { x: 0.0, y: 0.0, w: W, h: SCREEN_H };

    #[test]
    fn the_map_and_its_readout_fit_the_window() {
        let m = area(SC);
        assert!(m.x - 6.0 > SC.x && m.x + m.w + 6.0 < SC.x + SC.w);
        assert!(m.y - 6.0 > SC.y + 18.0, "map under the label");
        assert!(m.y + m.h + 22.0 < SC.y + SC.h, "no room for the readout");
    }

    #[test]
    fn every_room_lands_on_the_map() {
        let m = area(SC);
        for i in 0..SPACES {
            let (x, y) = at(m, room(i));
            assert!(x >= m.x && x <= m.x + m.w && y >= m.y && y <= m.y + m.h);
        }
        assert!(!family_labels().is_empty());
    }
}
