//! THE MACHINE DIAGRAM — the living center:
//! dynamo's OWN picture through the open scene kind, a
//! pure function of five probes. The boiler's needle droops under
//! chords, the rotor's spoke strobes with the shaft, the wires
//! wave with their energy, the bridge block glows with rail load.
//! Nothing here is decoration: every moving part IS a probe the
//! engine publishes as declared taps (never name-sniffed).
//!
//! Drawn into a face `Display` (see `draw`).
//!
//! Probe order: spin (rate/commanded 0..1), shaft (revolutions
//! 0..1), glow (ensemble wire energy 0..1), boiler (pressure
//! 0..1), rail (bridge |load| 0..1).

use stave_core::face::Frame;
use stave_ui::layout::Rect;
use stave_ui::scene::{pt, Font, Op, Path, Pt, Scene, Stroke, TextAlign};
use stave_ui::style::Color;

extern crate alloc;
use alloc::vec::Vec;

/// The probes the picture reads, in [`MachineState`] order.
pub const PROBES: &[&str] = &["spin", "shaft", "glow", "boiler", "rail"];

/// The display's draw fn: the machine in the theme's phosphor, live
/// from the five taps.
pub fn draw(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    let ph = f.theme.phosphor;
    let st = MachineState {
        spin: f.probe("spin") as f32,
        shaft: f.probe("shaft") as f32,
        glow: f.probe("glow") as f32,
        boiler: f.probe("boiler") as f32,
        rail: f.probe("rail") as f32,
        x0: r.x,
        y0: r.y,
        w: r.w,
        h: r.h,
    };
    stave_core::face::screens::grid(f, r, s, 12, 4);
    let style = MachineStyle {
        accent: ph,
        ink: dim(ph, 0.7),
        line: dim(ph, 0.35),
        font: f.theme.type_.caption.align(TextAlign::Center),
        label_size: f.theme.type_.caption_size,
    };
    s.ops.extend(scene(&st, &style).ops);
}

pub(crate) struct MachineState {
    pub spin: f32,
    pub shaft: f32,
    pub glow: f32,
    pub boiler: f32,
    pub rail: f32,
    /// The screen's top-left (face coordinates).
    pub x0: f64,
    pub y0: f64,
    pub w: f64,
    pub h: f64,
}

pub(crate) struct MachineStyle {
    pub accent: Color,
    pub ink: Color,
    pub line: Color,
    pub font: Font,
    pub label_size: f64,
}

fn dim(c: Color, a: f64) -> Color {
    Color::rgba(c.r, c.g, c.b, a)
}

/// The picture, pure (the laws read its ops).
pub(crate) fn scene(st: &MachineState, style: &MachineStyle) -> Scene {
    let mut sc = Scene::new();
    // Drawn on an 80-pt-tall design, scaled to the screen.
    let k = (st.h / 80.0).max(0.5);
    let cy = st.y0 + st.h * 0.44;
    let label_y = st.y0 + st.h - 8.0;
    let at = |frac: f64| st.x0 + st.w * frac;
    let label = |sc: &mut Scene, x: f64, text: &str| {
        sc.ops.push(Op::Text {
            at: pt(x, label_y),
            text: alloc::string::String::from(text),
            size: style.label_size,
            color: style.ink,
            font: style.font,
        });
    };

    // THE BOILER — a gauge whose needle IS the pressure: full
    // pressure points up-right, a drooping boiler swings left.
    let bx = at(0.08);
    sc.stroke(
        Path::arc(
            pt(bx, cy),
            15.0 * k,
            core::f64::consts::PI * 0.8,
            core::f64::consts::PI * 2.2,
            false,
        ),
        Stroke::round(style.line, 1.4),
    );
    let angle = core::f64::consts::PI * (0.8 + 1.4 * f64::from(st.boiler));
    let needle = pt(bx + 12.0 * k * angle.cos(), cy + 12.0 * k * angle.sin());
    sc.stroke(
        Path::polyline(&[pt(bx, cy), needle]),
        Stroke::round(style.accent, 1.6),
    );
    label(&mut sc, bx, "BOILER");

    // THE ROTOR — the shaft's spoke strobes with rotation; the rim
    // brightens as it reaches commanded speed.
    let rx = at(0.26);
    let rim = dim(style.accent, 0.25 + 0.6 * f64::from(st.spin));
    sc.stroke(Path::circle(pt(rx, cy), 17.0 * k), Stroke::round(rim, 1.6));
    let a = core::f64::consts::TAU * f64::from(st.shaft);
    for spoke in 0..4 {
        let ang = a + core::f64::consts::PI * 0.5 * f64::from(spoke);
        sc.stroke(
            Path::polyline(&[pt(rx, cy), pt(rx + 14.0 * k * ang.cos(), cy + 14.0 * k * ang.sin())]),
            Stroke::round(dim(style.ink, 0.8), 1.2),
        );
    }
    label(&mut sc, rx, "ROTOR");

    // THE WIRES — the pair waves with its energy: amplitude and
    // glow are the ensemble's ring.
    let (wx0, wx1) = (at(0.36), at(0.70));
    let amp = 7.0 * k * f64::from(st.glow);
    let wire_glow = dim(style.accent, 0.3 + 0.65 * f64::from(st.glow));
    for (dy, phase) in [(-6.0 * k, 0.0), (6.0 * k, core::f64::consts::PI)] {
        let mut pts: Vec<Pt> = Vec::with_capacity(25);
        for i in 0..=24 {
            let t = i as f64 / 24.0;
            let x = wx0 + (wx1 - wx0) * t;
            let y = cy + dy + amp * (t * core::f64::consts::TAU * 2.0 + phase).sin();
            pts.push(pt(x, y));
        }
        sc.stroke(Path::polyline(&pts), Stroke::round(wire_glow, 1.5));
    }
    label(&mut sc, (wx0 + wx1) * 0.5, "WIRES");

    // THE BRIDGE — one block, glowing with the rail's load.
    let brx = at(0.78);
    sc.solid(
        Path::rect(pt(brx - 4.0 * k, cy - 16.0 * k), 8.0 * k, 32.0 * k),
        dim(style.accent, 0.15 + 0.8 * f64::from(st.rail)),
    );
    sc.stroke(
        Path::rect(pt(brx - 4.0 * k, cy - 16.0 * k), 8.0 * k, 32.0 * k),
        Stroke::new(style.line, 1.0),
    );
    label(&mut sc, brx, "BRIDGE");

    // OUT — the machine's voice leaves stage right.
    sc.stroke(
        Path::polyline(&[pt(brx + 8.0 * k, cy), pt(at(0.93), cy)]),
        Stroke::round(dim(style.ink, 0.7), 1.4),
    );
    sc.stroke(
        Path::polyline(&[
            pt(at(0.93) - 5.0, cy - 4.0),
            pt(at(0.93), cy),
            pt(at(0.93) - 5.0, cy + 4.0),
        ]),
        Stroke::round(dim(style.ink, 0.7), 1.4),
    );

    sc
}

#[cfg(test)]
mod properties {
    use super::*;
    use stave_ui::scene::Seg;

    fn st(boiler: f32, glow: f32) -> MachineState {
        MachineState {
            spin: 1.0,
            shaft: 0.0,
            glow,
            boiler,
            rail: 0.0,
            x0: 0.0,
            y0: 0.0,
            w: 600.0,
            h: 96.0,
        }
    }

    fn style() -> MachineStyle {
        MachineStyle {
            accent: Color::rgba(0.5, 0.7, 0.9, 1.0),
            ink: Color::rgba(0.6, 0.6, 0.7, 1.0),
            line: Color::rgba(0.3, 0.3, 0.4, 1.0),
            font: Font::default(),
            label_size: 7.5,
        }
    }

    /// The needle IS the pressure: full and empty point to the
    /// declared gauge extremes.
    #[test]
    fn the_needle_is_the_pressure() {
        let needle_x = |b: f32| {
            let sc = scene(&st(b, 0.0), &style());
            // Second stroke = the needle polyline; its endpoint.
            let Op::Stroke { path, .. } = &sc.ops[1] else {
                panic!("needle")
            };
            let Seg::LineTo(p) = path.segs[1] else {
                panic!("needle end")
            };
            p.x
        };
        let full = needle_x(1.0);
        let empty = needle_x(0.0);
        // Full pressure points RIGHT of the pivot; empty swings
        // LEFT past it.
        assert!(full > 600.0 * 0.08, "full needle right of pivot: {full}");
        assert!(
            empty < full - 10.0,
            "empty needle must swing left: {empty} vs {full}"
        );
    }

    /// Quiet wires are FLAT lines; energetic wires wave.
    #[test]
    fn the_wires_wave_with_energy() {
        let spread_of = |glow: f32| {
            let sc = scene(&st(1.0, glow), &style());
            let mut min = f64::MAX;
            let mut max = f64::MIN;
            for op in &sc.ops {
                if let Op::Stroke { path, .. } = op {
                    if path.segs.len() > 10 {
                        for seg in &path.segs {
                            if let Seg::LineTo(p) | Seg::MoveTo(p) = seg {
                                min = min.min(p.y);
                                max = max.max(p.y);
                            }
                        }
                    }
                }
            }
            max - min
        };
        let quiet = spread_of(0.0);
        let hot = spread_of(1.0);
        assert!(
            hot > quiet + 8.0,
            "energy must wave the wires: {quiet} → {hot}"
        );
    }

    /// The picture is pure.
    #[test]
    fn the_scene_is_pure() {
        let a = scene(&st(0.7, 0.4), &style());
        let b = scene(&st(0.7, 0.4), &style());
        assert_eq!(a, b);
    }
}
