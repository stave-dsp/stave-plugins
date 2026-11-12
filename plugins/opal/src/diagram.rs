//! The algorithm diagram — OPAL'S OWN picture, a pure function of
//! the wiring table, drawn into a face `Display` (bespoke visuals
//! live in the project). Operator/DX
//! players navigate by this picture — eight cramped tab labels were
//! never going to say "(3,4)→2→1".
//!
//! Layout: modulator DEPTH becomes vertical position — an op's row
//! is its longest modulation-chain distance from a carrier, carriers
//! on the bottom row. Ops draw as rounded squares (carriers filled
//! with the accent, modulators outlined), routes as vertical/elbow
//! polylines entering the consumer's top edge, and op 4's
//! self-feedback as a small loop arc on its box.

use stave::face::{step_of, Typography};
use stave::prelude::*;

use crate::engine::operators::{DIAGRAM_CARRIERS, DIAGRAM_MODS};

/// The display's draw fn: the algorithm (the one bind the display
/// declares, `reads` in the face) drawn as its wiring, in the
/// theme's phosphor. The wiring tables are the engine's own consts
/// (const-derived from WIRINGS — the picture cannot drift from the
/// sound).
pub fn draw(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    let n = DIAGRAM_CARRIERS.len();
    let algo = step_of(f.bound(0), n as u32) as usize;
    let algo = algo.min(n - 1);
    let mut mods = [0u8; 8];
    for (i, m) in mods.iter_mut().enumerate().take(4) {
        *m = DIAGRAM_MODS[algo * 4 + i];
    }
    screens::grid(f, r, s, 8, 4);
    let ph = f.theme.palette.phosphor;
    let (pad_x, pad_y) = (r.w * 0.12, 14.0);
    let sc = scene(
        &AlgoState {
            wiring: Wiring {
                mods,
                carriers: DIAGRAM_CARRIERS[algo],
                ops: 4,
                feedback_op: Some(3),
            },
            area: Rect {
                x: r.x + pad_x,
                y: r.y + pad_y,
                w: r.w - 2.0 * pad_x,
                h: r.h - 2.0 * pad_y,
            },
        },
        &AlgoStyle {
            accent: ph,
            ink: Color::rgba(ph.r, ph.g, ph.b, 0.85),
            line: Color::rgba(ph.r, ph.g, ph.b, 0.55),
            bg: f.theme.palette.screen,
            typography: f.theme.typography,
        },
    );
    s.ops.extend(sc.ops);
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct AlgoStyle {
    pub accent: Color,
    pub ink: Color,
    pub line: Color,
    /// Number ink on FILLED (carrier) boxes — the panel background,
    /// so digits punch through the accent.
    pub bg: Color,
    /// The op numbers set in its value face.
    pub typography: Typography,
}

/// One algorithm's wiring: `mods[i]` = bitmask of ops modulating op
/// i; bit i of `carriers` = op i reaches the output. Op indices are
/// 0-based here (op 1 = index 0, the convention of the tables).
#[derive(Copy, Clone, Debug)]
pub(crate) struct Wiring {
    pub mods: [u8; 8],
    pub carriers: u8,
    pub ops: u8,
    /// Which op carries self-feedback (drawn as a loop), if any.
    pub feedback_op: Option<u8>,
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct AlgoState {
    pub wiring: Wiring,
    /// Where the diagram draws.
    pub area: Rect,
}

/// Each op's row = longest chain distance to a carrier (carriers =
/// row 0). Exported for the laws.
pub(crate) fn depths(w: &Wiring) -> [u8; 8] {
    let n = w.ops.min(8) as usize;
    let mut depth = [0u8; 8];
    // Fixed-point over the DAG (mods reference higher ops only, but
    // stay general): depth(consumer) known → its mods sit above it.
    for _ in 0..n {
        for i in 0..n {
            for j in 0..n {
                if w.mods[i] & (1 << j) != 0 && depth[j] < depth[i] + 1 {
                    depth[j] = depth[i] + 1;
                }
            }
        }
    }
    depth
}

const BOX: f64 = 22.0;

/// Each op's box (center ± `BOX / 2`). Exported for the laws.
pub(crate) fn positions(st: &AlgoState) -> Vec<Rect> {
    let n = st.wiring.ops.min(8) as usize;
    let depth = depths(&st.wiring);
    let max_depth = (0..n).map(|i| depth[i]).max().unwrap_or(0);
    let rows = f64::from(max_depth) + 1.0;
    let a = st.area;
    let row_h = (a.h - BOX) / rows.max(2.0);
    (0..n)
        .map(|i| {
            // Column = op index, spread across the width; row by depth
            // (carriers at the bottom).
            let x = a.x + (i as f64 + 0.5) * a.w / n as f64;
            let y = a.y + a.h - BOX * 0.5 - f64::from(depth[i]) * row_h;
            Rect { x: x - BOX * 0.5, y: y - BOX * 0.5, w: BOX, h: BOX }
        })
        .collect()
}

pub(crate) fn scene(st: &AlgoState, style: &AlgoStyle) -> Scene {
    let mut sc = Scene::new();
    let n = st.wiring.ops.min(8) as usize;
    let pos = positions(st);

    // Routes first (under the boxes): consumer top edge ← mod bottom.
    for i in 0..n {
        for j in (i + 1)..n {
            if st.wiring.mods[i] & (1 << j) != 0 {
                let (m, c) = (pos[j], pos[i]);
                let from = pt(m.x + m.w * 0.5, m.y + m.h);
                let to = pt(c.x + c.w * 0.5, c.y);
                let mid_y = (from.y + to.y) * 0.5;
                sc.stroke(
                    Path::polyline(&[from, pt(from.x, mid_y), pt(to.x, mid_y), to]),
                    Stroke::new(style.line, 1.2),
                );
            }
        }
    }
    // The boxes: carriers filled, modulators outlined — each
    // NUMBERED so the picture answers "which op is that?" (the
    // numbers match the OP 1–4 pages; the diagram was unreadable
    // anonymous).
    for (i, &b) in pos.iter().enumerate() {
        let is_carrier = st.wiring.carriers & (1 << i) != 0;
        let boxp = Path::rect(pt(b.x, b.y), b.w, b.h);
        let num_ink = if is_carrier {
            sc.solid(boxp, style.accent);
            style.bg
        } else {
            sc.stroke(boxp, Stroke::new(style.ink, 1.2));
            style.ink
        };
        let at = pt(b.x + b.w * 0.5, b.y + b.h * 0.5 + 3.4);
        sc.text_in(at, format!("{}", i + 1), 10.0, num_ink, style.typography.value);
    }
    // The self-feedback loop.
    if let Some(fb) = st.wiring.feedback_op {
        let i = fb.min(7) as usize;
        if i < n {
            let b = pos[i];
            sc.stroke(
                Path::arc(pt(b.x + b.w, b.y), 5.0, 0.0, 4.7, false),
                Stroke::new(style.line, 1.2),
            );
        }
    }
    sc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style() -> AlgoStyle {
        AlgoStyle {
            accent: Color::rgba(0.4, 0.7, 1.0, 1.0),
            ink: Color::rgba(0.6, 0.65, 0.75, 1.0),
            line: Color::rgba(0.35, 0.38, 0.45, 1.0),
            bg: Color::rgba(0.06, 0.07, 0.08, 1.0),
            typography: Theme::default().typography,
        }
    }

    /// The scene's ops as their debug text: `Op` is not reachable from
    /// `stave` (only `Scene` is), so the laws read the ops' shapes off
    /// their `Debug` form.
    fn ops(sc: &Scene) -> Vec<String> {
        sc.ops.iter().map(|o| format!("{o:?}")).collect()
    }

    /// A stroke op's width and segment count.
    fn stroke_of(op: &str) -> Option<(bool, usize)> {
        let body = op.strip_prefix("Stroke {")?;
        let segs = ["MoveTo(", "LineTo(", "QuadTo(", "CubicTo(", "Arc {", "Close"]
            .iter()
            .map(|s| body.matches(s).count())
            .sum();
        Some((body.contains("width: 1.2"), segs))
    }

    fn chain() -> Wiring {
        // 4→3→2→1, carrier 1, feedback on 4 (opal's CHAIN).
        Wiring {
            mods: [0b0010, 0b0100, 0b1000, 0, 0, 0, 0, 0],
            carriers: 0b0001,
            ops: 4,
            feedback_op: Some(3),
        }
    }

    fn organ() -> Wiring {
        Wiring {
            mods: [0; 8],
            carriers: 0b1111,
            ops: 4,
            feedback_op: Some(3),
        }
    }

    fn st(w: Wiring) -> AlgoState {
        AlgoState {
            wiring: w,
            area: Rect { x: 0.0, y: 0.0, w: 200.0, h: 120.0 },
        }
    }

    /// The diagram's route count IS the wiring table's popcount —
    /// the acceptance checks.
    #[test]
    fn route_count_equals_the_tables_popcount() {
        for w in [chain(), organ()] {
            let expected: u32 = w.mods.iter().map(|m| m.count_ones()).sum();
            let sc = scene(&st(w), &style());
            let routes = ops(&sc)
                .iter()
                .filter(|op| stroke_of(op) == Some((true, 4)))
                .count() as u32;
            assert_eq!(routes, expected);
        }
    }

    /// The CHAIN stacks four rows deep; the ORGAN is one flat row.
    #[test]
    fn depth_reads_the_topology() {
        let d = depths(&chain());
        assert_eq!(
            &d[..4],
            &[0, 1, 2, 3],
            "chain: each op one row above its consumer"
        );
        assert_eq!(
            &depths(&organ())[..4],
            &[0, 0, 0, 0],
            "organ: all carriers, one row"
        );
    }

    /// Carriers fill, modulators outline — countable from the ops.
    #[test]
    fn carriers_fill_modulators_outline() {
        let sc = scene(&st(chain()), &style());
        let fills = ops(&sc).iter().filter(|o| o.starts_with("Fill {")).count();
        assert_eq!(fills, 1, "CHAIN has exactly one carrier");
        let sc = scene(&st(organ()), &style());
        let fills = ops(&sc).iter().filter(|o| o.starts_with("Fill {")).count();
        assert_eq!(fills, 4, "ORGAN is all carriers");
    }

    /// Every op box carries its own number — 1..=N, in op order.
    #[test]
    fn every_box_is_numbered() {
        let sc = scene(&st(chain()), &style());
        let nums: Vec<_> = sc
            .ops
            .iter()
            .filter_map(|o| {
                let o = format!("{o:?}");
                let rest = o.strip_prefix("Text {")?.split("text: \"").nth(1)?;
                Some(rest.split('"').next()?.to_owned())
            })
            .collect();
        assert_eq!(nums, ["1", "2", "3", "4"]);
    }

    #[test]
    fn the_scene_is_pure() {
        assert_eq!(scene(&st(chain()), &style()), scene(&st(chain()), &style()));
    }
}
