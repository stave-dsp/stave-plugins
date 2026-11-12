//! The dynamo face — the whole window, a pure function of params, on
//! machine-green hammertone (a boiler-room instrument, brass
//! needles and amber gauges).
//!
//! ```text
//! header   DYNAMO · MACHINE/VOICE/ROUTES/MOTION pager · bypass
//! MACHINE  the living diagram (five engine taps) over the six
//!          macros, each with its MOTION (source + depth of the
//!          pre-wired route slot 5–10 that drives it)
//! VOICE    ENVELOPE · ROTOR / WIRES · LIFE
//! ROUTES   LFO · the four free route slots (rows 1–4)
//! MOTION   the six motion slots in full (rows 5–10)
//! ```

use stave::face::photo::hex;
use stave::face::{Finish, Hardware, Palette};
use stave::params::AnyParam;
use stave::prelude::*;
use stave::route::RouteParams;

use crate::engine::DynamoTaps;
use crate::params::DynamoParams;

/// The editor-local page set — view state, never a param.
const PAGES: u32 = 1;

/// Dynamo's finish: machine-green hammertone, black bakelite knobs,
/// brass accent, amber gauges.
fn theme() -> Theme {
    let base = Theme::hammertone();
    Theme {
        name: "dynamo",
        finish: Finish {
            surface: hex(0x4B5546),
            plate: hex(0x3E473A),
            desk: hex(0x1F201C),
            ..base.finish
        },
        palette: Palette {
            ink: Color::rgba(0.93, 0.90, 0.80, 1.0),
            ink_soft: Color::rgba(0.93, 0.90, 0.80, 0.58),
            accent: Color::rgba(0.96, 0.69, 0.27, 1.0),
            screen: Color::rgba(0.05, 0.04, 0.02, 1.0),
            phosphor: Color::rgba(1.0, 0.72, 0.32, 1.0),
        },
        hardware: Hardware {
            lamp: [1.8, 0.62, 0.14],
            cap: hex(0x141412),
            pointer: Color::rgba(0.95, 0.93, 0.86, 1.0),
            ..base.hardware
        },
        ..base
    }
}

/// The ADSR picture, from the four binds the display declares
/// (attack, decay, sustain, release — `reads` in `face`).
fn envelope(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    screens::envelope(f, r, s, f.bound(0), f.bound(1), f.bound(2), f.bound(3));
}

/// One MACRO column (the Atoms lesson): the macro knob with its
/// MOTION directly beneath — source + depth of the pre-wired slot
/// that targets this macro.
fn macro_col<S: ParamEnum, T: ParamEnum>(knob: &dyn AnyParam, motion: &RouteParams<S, T>) -> FaceNode {
    column(vec![
        Knob::new(knob.bind()).large().node(),
        Stepper::new(motion.src.bind()).label("MOTION").width(108.0).node(),
        Knob::new(motion.depth.bind()).small().centered().label("DEPTH").node(),
    ])
    .gap(10.0)
    .align(Align::Center)
}

const LABELS: [&str; 10] = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"];

fn routes_plate(p: &DynamoParams, title: &'static str, slots: core::ops::Range<usize>, gap: f64) -> FaceNode {
    let rows: Vec<FaceNode> = slots.map(|i| p.routes[i].row(DynamoTaps::MODS[i], LABELS[i])).collect();
    Plate::titled(title).around(column(rows).gap(gap).align(Align::Center))
}

pub fn face(p: &DynamoParams) -> Face {
    let theme = theme();
    let gap = theme.space(3);

    let header = row(vec![
        Title::new("DYNAMO").tagline("the impossible machine").node(),
        Pager::new(PAGES, &["MACHINE", "VOICE", "ROUTES", "MOTION"]).node(),
        Footswitch::bypass(p.bypass.bind()).compact().node(),
    ])
    .gap(theme.space(4))
    .align(Align::Center);

    // ---- MACHINE -------------------------------------------------
    let macros: [&dyn AnyParam; 6] = [&p.force, &p.mass, &p.memory, &p.couple, &p.steam, &p.chaos];
    let machine = Plate::titled("MACHINE").around(
        column(vec![
            Display::new(820.0, 132.0, crate::machine::draw).taps(&crate::machine::TAPS).animated().node(),
            row(macros.iter().enumerate().map(|(i, k)| macro_col(*k, &p.routes[4 + i])).collect())
                .gap(gap + 8.0)
                .align(Align::Start),
        ])
        .gap(theme.space(4))
        .align(Align::Center),
    );

    // ---- VOICE ---------------------------------------------------
    let env = Plate::titled("ENVELOPE").around(
        column(vec![
            Display::new(400.0, 104.0, envelope)
                .reads(&[p.attack.bind(), p.decay.bind(), p.sustain.bind(), p.release.bind()])
                .node(),
            row(vec![
                Knob::new(p.attack.bind()).large().node(),
                Knob::new(p.decay.bind()).large().node(),
                Knob::new(p.sustain.bind()).large().node(),
                Knob::new(p.release.bind()).large().node(),
            ])
            .gap(theme.space(4)),
        ])
        .gap(gap)
        .align(Align::Center),
    );
    let rotor = Plate::titled("ROTOR").around(
        column(vec![
            row(vec![Knob::new(p.sag.bind()).large().node(), Toggle::new(p.contact.bind()).node()])
                .gap(theme.space(4))
                .align(Align::Center),
            row(vec![Selector::new(p.poles.bind()).node(), Stepper::new(p.wheel.bind()).width(124.0).node()])
                .gap(gap)
                .align(Align::Center),
        ])
        .gap(gap)
        .align(Align::Center),
    );
    let wires = Plate::titled("WIRES").around(
        row(vec![
            Knob::new(p.ring.bind()).large().node(),
            Knob::new(p.stiff.bind()).large().node(),
            Knob::new(p.width.bind()).large().node(),
            Knob::new(p.output.bind()).large().centered().node(),
        ])
        .gap(8.0)
        .align(Align::Center),
    );
    let life = Plate::titled("LIFE").around(
        row(vec![
            Knob::new(p.hum.bind()).node(),
            Knob::new(p.bearing.bind()).node(),
            Knob::new(p.clank.bind()).node(),
            Knob::new(p.crackle.bind()).node(),
            Knob::new(p.tension.bind()).node(),
            Knob::new(p.grit.bind()).node(),
        ])
        .gap(0.0)
        .align(Align::Center),
    );
    let voice_page = column(vec![
        row(vec![env, rotor]).gap(gap).align(Align::Stretch),
        row(vec![wires, life]).gap(gap).align(Align::Stretch),
    ])
    .gap(gap)
    .align(Align::Stretch);

    // ---- ROUTES / MOTION ----------------------------------------
    let lfo = Plate::titled("LFO").around(
        row(vec![
            Knob::new(p.lfo_rate.bind()).label("RATE").node(),
            Knob::new(p.lfo_rise.bind()).label("RISE").node(),
            Selector::new(p.lfo_shape.bind()).label("SHAPE").node(),
            Selector::new(p.lfo_sync.bind()).label("SYNC").node(),
            Toggle::new(p.lfo_trig.bind()).label("TRIG").node(),
        ])
        .gap(gap)
        .align(Align::Center),
    );
    // Six motion rows share one page: strips sit closer than plates.
    let strip = theme.space(0);
    let routes_page = column(vec![lfo, routes_plate(p, "ROUTES", 0..4, strip)]).gap(gap).align(Align::Stretch);
    let motion_page = routes_plate(p, "MOTION · ONE PER MACRO", 4..10, strip);

    let pages = pages(PAGES, vec![machine, voice_page, routes_page, motion_page]);
    let body = column(vec![header, pages]).gap(theme.space(4)).align(Align::Stretch);
    Face::new(theme, Panel::default().around(body))
}
