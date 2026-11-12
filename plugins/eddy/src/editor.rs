//! The eddy face — the whole window, a pure function of params.
//! Organized by SCOPE: the line every algorithm regenerates through
//! (DELAY), its levels and the tail picture stay on top, always; the
//! three deeper sections — MOTION (the global wobble), ALGORITHM (the
//! feedback law and every law's own knobs) and DIFFUSION (the field's
//! voice, its own knobs, and the couplings) — sit behind a pager.
//! On the ALGO and FIELD pages a row of tabs picks what runs (the
//! `algo` and `diffusion` params, so a tab automates and saves like a
//! knob), and the plate under it follows the tab: only the running
//! law's, and the running voice's, own knobs are on the face.

use stave::face::photo::hex;
use stave::face::{CapStyle, Coat, Finish, Hardware, Palette};
use stave::prelude::*;

use crate::engine::EddyTaps;
use crate::params::EddyParams;

/// The editor-local page set behind the pager.
const PAGES: u32 = 1;

/// Mustard powder coat, black print, black dome knobs, red light.
fn theme() -> Theme {
    Theme {
        name: "eddy",
        finish: Finish {
            coat: Coat::Powder,
            surface: hex(0xB9862A),
            plate: hex(0xA67624),
            desk: hex(0x1F1D1B),
            radius: 14.0,
        },
        palette: Palette {
            ink: Color::rgba(0.11, 0.09, 0.07, 1.0),
            ink_soft: Color::rgba(0.11, 0.09, 0.07, 0.62),
            accent: Color::rgba(0.62, 0.12, 0.08, 1.0),
            screen: Color::rgba(0.05, 0.04, 0.03, 1.0),
            phosphor: Color::rgba(1.0, 0.78, 0.38, 1.0),
        },
        hardware: Hardware {
            caps: CapStyle::Dome,
            cap: hex(0x0F0E0D),
            pointer: Color::rgba(0.96, 0.94, 0.9, 1.0),
            lamp: [1.9, 0.3, 0.1],
        },
        ..Theme::graphite()
    }
}

/// The motion cycle the engine publishes (one LFO period, ±1), with a
/// lit cursor where the read head is now.
fn wave(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    screens::grid(f, r, s, 8, 4);
    let ys = f.controller.curve();
    if ys.len() < 2 {
        return;
    }
    let inner = Rect { x: r.x + 8.0, y: r.y + 10.0, w: r.w - 16.0, h: r.h - 20.0 };
    let at = |i: usize| {
        let u = i as f64 / (ys.len() - 1) as f64;
        let v = f64::from(ys[i]).clamp(-1.0, 1.0);
        pt(inner.x + u * inner.w, inner.y + inner.h * 0.5 * (1.0 - v))
    };
    let pts: Vec<_> = (0..ys.len()).map(at).collect();
    screens::glow_line(f, s, &pts);
    let phase = f.tap(EddyTaps::LFO_PHASE).clamp(0.0, 1.0);
    let c = at(((ys.len() - 1) as f64 * phase).round() as usize);
    let ph = f.theme.palette.phosphor;
    s.solid(Path::circle(c, 7.0), Color::rgba(ph.r, ph.g, ph.b, 0.22));
    s.solid(Path::circle(c, 3.2), ph);
}

pub fn face(p: &EddyParams) -> Face {
    let theme = theme();
    let gap = theme.space(3);
    let wide = theme.space(4);

    let header = row(vec![
        Title::new("EDDY").tagline("a feedback delay whose read head breathes").node(),
        spacer(),
        Presets::default().node(),
        Footswitch::bypass(p.bypass.bind()).compact().node(),
    ])
    .gap(wide)
    .align(Align::Center);

    // Always on top: the line, its levels, its tail.
    let delay = Plate::titled("DELAY").around(
        row(vec![Knob::new(p.time.bind()).huge().node(), Knob::new(p.feedback.bind()).huge().node()])
            .gap(wide)
            .align(Align::End),
    );
    let levels = Plate::titled("LEVELS").around(
        row(vec![
            Knob::new(p.dry.bind()).node(),
            Knob::new(p.wet.bind()).node(),
            Knob::new(p.output.bind()).centered().node(),
        ])
        .gap(gap),
    );
    let tail = Plate::titled("TAIL").around(Trace::new(EddyTaps::TAIL, 236.0, 92.0, (0.0, 1.0)).node());
    let top = row(vec![delay, levels, tail]).gap(gap).align(Align::Stretch);

    // MOTION: the wobble's picture, its character and its knobs.
    let cycle = Plate::titled("CYCLE").around(Display::new(300.0, 214.0, wave).taps(&[EddyTaps::LFO_PHASE]).node());
    let motion = Plate::titled("MOTION").around(
        column(vec![
            Selector::new(p.motion.bind()).label("SHAPE").node(),
            row(vec![
                Knob::new(p.rate.bind()).node(),
                Knob::new(p.depth.bind()).node(),
                Knob::new(p.spread.bind()).node(),
                Knob::new(p.flavor.bind()).node(),
            ])
            .gap(wide),
        ])
        .gap(gap)
        .align(Align::Center),
    );
    let motion_page = row(vec![cycle, motion]).gap(gap).align(Align::Stretch);

    // ALGORITHM: the tabs pick the feedback law; the plate under them
    // is that law's own knobs.
    let circuit = Plate::titled("CIRCUIT").around(
        row(vec![Toggle::new(p.circuit.bind()).label("CELL").node(), Knob::new(p.heat.bind()).node()])
            .gap(gap)
            .align(Align::Center),
    );
    let motor = Plate::titled("MOTOR")
        .around(row(vec![Knob::new(p.sag.bind()).node(), Knob::new(p.inertia.bind()).node()]).gap(gap));
    let shift = Plate::titled("SHIFT").around(
        row(vec![
            Knob::new(p.shift.bind()).centered().node(),
            Knob::new(p.grain.bind()).node(),
            Knob::new(p.splice.bind()).node(),
        ])
        .gap(gap),
    );
    let orbit = Plate::titled("ORBIT").around(Knob::new(p.orbit.bind()).centered().node());
    let algo_page = column(vec![
        Tabs::of(&p.algo).node(),
        follow(p.algo.bind(), vec![circuit, motor, shift, orbit]),
    ])
    .gap(gap)
    .align(Align::Stretch);

    // DIFFUSION: the tabs pick the field's voice; beside its knobs,
    // the couplings every voice shares.
    let couple = Plate::titled("COUPLING").around(
        row(vec![
            Knob::new(p.diffuse.bind()).node(),
            Knob::new(p.hold.bind()).node(),
            Knob::new(p.bleed.bind()).node(),
        ])
        .gap(gap),
    );
    let pair = |title: &'static str, a: Bind, b: Bind| {
        Plate::titled(title).around(row(vec![Knob::new(a).node(), Knob::new(b).node()]).gap(gap))
    };
    let voices = follow(
        p.diffusion.bind(),
        vec![
            pair("PLATE", p.density.bind(), p.wander.bind()),
            pair("PRISM", p.bend.bind(), p.smear.bind()),
            pair("DUST", p.spray.bind(), p.motes.bind()),
            pair("MESH", p.size.bind(), p.weave.bind()),
        ],
    );
    let field_page = column(vec![
        Tabs::of(&p.diffusion).node(),
        row(vec![voices, couple]).gap(gap).align(Align::Stretch),
    ])
    .gap(gap)
    .align(Align::Stretch);

    let pages = stack(vec![
        on_page(motion_page.width(Dim::Fill(1.0)), PAGES, 0),
        on_page(algo_page.width(Dim::Fill(1.0)), PAGES, 1),
        on_page(field_page.width(Dim::Fill(1.0)), PAGES, 2),
    ]);

    let body = column(vec![
        header,
        top,
        row(vec![Pager::new(PAGES, &["MOTION", "ALGO", "FIELD"]).node().id("pages"), spacer()]),
        pages,
    ])
    .gap(theme.space(3))
    .align(Align::Stretch);
    Face::new(theme, Panel { margin: 18.0, pad: 24.0, ..Panel::default() }.around(body))
}

/// `plates[i]` while `bind` is on step `i`, overlaid and each stretched
/// to the largest, so switching never moves the rest of the face.
fn follow(bind: Bind, plates: Vec<FaceNode>) -> FaceNode {
    let steps = plates
        .into_iter()
        .zip(0..)
        .map(|(n, i)| on_step(n.width(Dim::Fill(1.0)).height(Dim::Fill(1.0)), bind, i))
        .collect();
    stack(steps)
}
