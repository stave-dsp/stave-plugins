//! The MEMORY window: the bank (8 × 8 cells, held and attended), what
//! the ear HEARS (twelve pitch classes and the PULSE lamp), and the
//! RECALL and NOVELTY strips.

use stave::prelude::*;

use super::{glow, label, print, unlit, Feed, SLOTS};

/// The window's screen width (its height is [`super::SCREEN_H`]).
pub const W: f64 = 238.0;

const PAD: f64 = 14.0;
const COLS: usize = 8;
const PITCH: f64 = 17.5;
const CELL: f64 = 13.5;
const GRID_TOP: f64 = 30.0;
/// The hearing column starts this far right of the grid.
const HEAR_GAP: f64 = 30.0;
const STRIP_Y: [f64; 2] = [188.0, 205.0];
const STRIP_X0: f64 = 64.0;
const STRIP_SEGS: usize = 22;
const NOTES: [&str; 12] = ["C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B"];

/// Memory cell `k`'s rect.
pub fn cell(r: Rect, k: usize) -> Rect {
    let (c, row) = ((k % COLS) as f64, (k / COLS) as f64);
    Rect { x: r.x + PAD + c * PITCH, y: r.y + GRID_TOP + row * PITCH, w: CELL, h: CELL }
}

fn grid_w() -> f64 {
    (COLS - 1) as f64 * PITCH + CELL
}

/// Hearing bar `c`'s slot (full length).
pub fn hear_slot(r: Rect, c: usize) -> Rect {
    let x = r.x + PAD + grid_w() + HEAR_GAP;
    let pitch = grid_w() / 12.0;
    Rect { x, y: r.y + GRID_TOP + c as f64 * pitch + 1.0, w: r.x + r.w - PAD - x, h: pitch - 3.5 }
}

/// Strip `k`'s segment `s`.
pub fn strip_segment(r: Rect, k: usize, s: usize) -> Rect {
    let (x0, x1) = (r.x + STRIP_X0, r.x + r.w - PAD);
    let pitch = (x1 - x0) / STRIP_SEGS as f64;
    Rect { x: x0 + s as f64 * pitch, y: r.y + STRIP_Y[k] - 3.5, w: pitch - 2.2, h: 7.0 }
}

fn pulse_at(r: Rect) -> (f64, f64) {
    let h = hear_slot(r, 0);
    (h.x + h.w - 3.0, r.y + 16.0)
}

fn rect(s: &mut Scene, r: Rect, c: Color) {
    s.solid(Path::rect(pt(r.x, r.y), r.w, r.h), c);
}

fn halo(s: &mut Scene, r: Rect, by: f64, c: Color) {
    s.solid(Path::rect(pt(r.x - by, r.y - by), r.w + 2.0 * by, r.h + 2.0 * by), c);
}

/// The picture (a `Display` draw fn).
pub fn draw(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    let th = f.theme;
    let feed = Feed::read(f);
    let off = unlit(th);

    // MEMORY: the bank, cell by cell — held, and attended.
    label(f, s, (r.x + PAD + grid_w() / 2.0, r.y + 18.0), "MEMORY", 0.6, TextAlign::Center);
    for k in 0..SLOTS {
        let c = cell(r, k);
        if !feed.held(k) {
            rect(s, c, off);
            continue;
        }
        let a = feed.attention(k);
        if a > 0.02 {
            halo(s, c, 2.5, feed.ink(th, k, 0.25 * a));
        }
        rect(s, c, feed.ink(th, k, 0.18 + 0.82 * a));
    }

    // HEARING: the query's twelve pitch classes, dimmed as what you play
    // turns percussive (harmony then counts for less); PULSE on a hit.
    let h0 = hear_slot(r, 0);
    label(f, s, (h0.x - 10.0, r.y + 18.0), "HEARING", 0.6, TextAlign::Left);
    let p = pulse_at(r);
    let p = pt(p.0, p.1);
    s.solid(Path::circle(p, 2.2), super::amber(0.08));
    if feed.pulse > 0.05 {
        s.solid(Path::circle(p, 5.0), super::amber(0.3 * feed.pulse));
        s.solid(Path::circle(p, 2.2), super::amber(0.4 + 0.6 * feed.pulse));
    }
    let tonal = 0.3 + 0.7 * feed.tonal;
    for (c, note) in NOTES.iter().enumerate() {
        let slot = hear_slot(r, c);
        rect(s, slot, off);
        let v = feed.hearing.get(c).copied().unwrap_or(0.0);
        if v > 0.02 {
            rect(s, Rect { w: slot.w * v, ..slot }, glow(th, (0.35 + 0.65 * v) * tonal));
        }
        print(th, s, (slot.x - 4.0, slot.y + slot.h - 0.5), *note, 5.5, glow(th, 0.45), TextAlign::Right);
    }

    // RECALL (how loudly it remembers) and NOVELTY (how new this is).
    for (k, (value, name)) in [(feed.recalling, "RECALL"), (feed.novelty, "NOVELTY")].iter().enumerate() {
        label(f, s, (r.x + PAD, r.y + STRIP_Y[k] + 2.5), *name, 0.75, TextAlign::Left);
        let e = value * STRIP_SEGS as f64;
        for seg in 0..STRIP_SEGS {
            let at = strip_segment(r, k, seg);
            let on = (e - seg as f64).clamp(0.0, 1.0);
            if on > 0.05 {
                halo(s, at, 1.5, glow(th, 0.22 * on));
                rect(s, at, glow(th, 0.35 + 0.65 * on));
            } else {
                rect(s, at, off);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vfd::SCREEN_H;

    const R: Rect = Rect { x: 0.0, y: 0.0, w: W, h: SCREEN_H };

    fn inside(a: Rect) -> bool {
        a.x >= R.x + 4.0 && a.y >= R.y + 4.0 && a.x + a.w <= R.x + R.w - 4.0 && a.y + a.h <= R.y + R.h - 4.0
    }

    #[test]
    fn the_panels_fit_the_window_and_do_not_collide() {
        assert!(inside(cell(R, 0)) && inside(cell(R, SLOTS - 1)));
        assert!(inside(hear_slot(R, 0)) && inside(hear_slot(R, 11)));
        assert!(inside(strip_segment(R, 1, STRIP_SEGS - 1)));
        let grid_r = cell(R, COLS - 1);
        assert!(grid_r.x + grid_r.w + 12.0 < hear_slot(R, 0).x, "grid / hearing overlap");
        assert!(hear_slot(R, 0).w > 30.0, "hearing bars too short to read");
        let last = cell(R, SLOTS - 1);
        assert!(last.y + last.h + 8.0 < strip_segment(R, 0, 0).y, "grid / strips overlap");
    }
}
