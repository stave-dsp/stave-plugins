//! The tolex face — the whole window, a pure function of params: a
//! speaker cabinet. Black tolex around a wheat grille with the badge;
//! the control strip below reads the response (the model over the
//! reference population, and the cone's travel), then the build in
//! signal order: SPEAKER → AMP, CAB → MIC → OUTPUT.

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use stave_core::face::prelude::*;
use stave_core::face::{CapStyle, Finish};
use stave_core::params::BindParam;
use stave_ui::photo::hex;

use crate::params::TolexParams;
use crate::response;

/// Tolex's look: black tolex, cream piping, a wheat grille, cream
/// chicken heads on a dark crinkle strip, amber glass.
pub fn theme() -> Theme {
    let base = Theme::graphite();
    Theme {
        name: "tolex",
        surface: hex(0x242220),
        finish: Finish::Crinkle,
        plate: hex(0x1B1A19),
        desk: hex(0x191817),
        ink: Color::rgba(0.94, 0.90, 0.80, 1.0),
        ink_soft: Color::rgba(0.94, 0.90, 0.80, 0.55),
        accent: Color::rgba(1.0, 0.72, 0.30, 1.0),
        caps: CapStyle::Chicken,
        cap: hex(0xE8DFC8),
        tolex: hex(0x141312),
        piping: hex(0xDCD2BA),
        grille: hex(0x5A4A36),
        badge: [0.93, 0.89, 0.80],
        phosphor: Color::rgba(1.0, 0.74, 0.38, 1.0),
        screen: Color::rgba(0.05, 0.04, 0.03, 1.0),
        ..base
    }
}

fn group(title: &'static str, parts: Vec<FaceNode>, gap: f64) -> FaceNode {
    Plate::titled(title).around(row(parts).gap(gap).align(Align::Center))
}

pub fn face(p: &TolexParams) -> Face {
    let theme = theme();
    let gap = theme.space(2);
    let row_gap = theme.space(3);

    let view = row(vec![
        Display::new(470.0, 104.0, response::draw)
            .probes(&["excursion"])
            .reads(&response::reads(p))
            .node(),
        Trace::new("travel", 120.0, 108.0, (0.0, 1.0))
            .label("CONE TRAVEL")
            .node(),
        column(vec![
            Presets { width: 176.0 }.node(),
            row(vec![Footswitch::bypass(p.bypass.bind()).compact().node()])
                .justify(Justify::Center),
        ])
        .gap(0.0)
        .align(Align::Center),
    ])
    .gap(row_gap)
    .align(Align::Center);

    let speaker = group(
        "SPEAKER",
        vec![
            Selector::new(p.size.bind()).node(),
            Knob::new(p.resonance.bind()).node(),
            Knob::new(p.q.bind()).node(),
            Knob::new(p.cone.bind()).node(),
            Knob::new(p.mass.bind()).node(),
            Knob::new(p.breakup.bind()).node(),
            Knob::new(p.doping.bind()).node(),
        ],
        gap,
    );
    let amp = group(
        "AMP",
        vec![
            Knob::new(p.drive.bind()).node(),
            Knob::new(p.damping.bind()).node(),
        ],
        gap,
    );
    let cab = group(
        "CAB",
        vec![
            Toggle::new(p.back.bind()).node(),
            Toggle::new(p.material.bind()).node(),
            Toggle::new(p.speakers.bind()).node(),
            Knob::new(p.cab.bind()).node(),
        ],
        gap,
    );
    let mic = group(
        "MIC",
        vec![
            Stepper::new(p.model.bind()).width(112.0).node(),
            Knob::new(p.mic.bind()).label("DISTANCE").node(),
            Knob::new(p.position.bind()).node(),
        ],
        gap,
    );
    let output = group(
        "OUTPUT",
        vec![Knob::new(p.output.bind()).centered().node()],
        gap,
    );

    let body = column(vec![
        view,
        row(vec![speaker, amp]).gap(row_gap).align(Align::Stretch),
        row(vec![cab, mic, output])
            .gap(row_gap)
            .align(Align::Stretch),
    ])
    .gap(row_gap)
    .align(Align::Stretch);
    let panel = Panel {
        margin: 10.0,
        ..Panel::cabinet().grille(Side::Top, 140.0, Some("Tolex"))
    };
    Face::new(theme, panel.around(body))
}
