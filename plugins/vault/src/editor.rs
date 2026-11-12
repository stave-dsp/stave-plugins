//! The vault face — the whole window, a pure function of params.
//!
//! THE MACHINE spine + THE VOICE: every shared dial lives in exactly
//! one place, always visible, and the machine reinterprets it (the
//! Valhalla lesson). The machine window says what the shared dials
//! MEAN on the machine that runs. SPACE is where you are, ENERGY is
//! how the place holds sound, OUTPUT is the plumbing. Along the
//! bottom, the running machine's own plate — what exists on one
//! machine and not the others, named for it: the plate follows the
//! topology selector (`on_step`), one machine's plate at a time.

use stave::face::photo::hex;
use stave::face::{step_of, CapStyle, Coat, Finish, Hardware, Palette};
use stave::prelude::*;

use crate::params::{VaultParams, TOPOLOGIES};

/// What the shared dials mean on each machine, one line per dial.
const LEGENDS: [[&str; 4]; 5] = [
    [
        "LOW          how long the bottom rings",
        "HIGH         absorption, compounds per bounce",
        "TONE         the walls' finish",
        "DIFFUSE      scatter",
    ],
    [
        "LOW          the bottom's ring",
        "HIGH         the damper",
        "TONE         the pickup",
        "DIFFUSE      strike smear",
    ],
    [
        "DECAY        how long the memory holds",
        "LOW / HIGH   bend the hold",
        "TONE         its brightness",
        "DIFFUSE      soften the strike",
    ],
    [
        "DECAY        the burst's length",
        "LOW          the burst's bottom",
        "TONE         brightness",
        "DIFFUSE      density",
    ],
    [
        "DECAY        the field's hold",
        "LOW / HIGH   bend it",
        "TONE         brightness",
        "DIFFUSE      the wash",
    ],
];

/// Deep aubergine enamel, gilt print, black dome knobs, a violet glow.
fn theme() -> Theme {
    let gilt = Color::rgba(0.91, 0.82, 0.62, 1.0);
    let base = Theme::graphite();
    Theme {
        name: "vault",
        finish: Finish {
            coat: Coat::Enamel,
            surface: hex(0x2B1E3A),
            plate: hex(0x22172F),
            desk: hex(0x141218),
            radius: 14.0,
        },
        palette: Palette {
            ink: gilt,
            ink_soft: Color::rgba(0.91, 0.82, 0.62, 0.55),
            accent: Color::rgba(0.86, 0.72, 1.0, 1.0),
            screen: Color::rgba(0.03, 0.02, 0.05, 1.0),
            phosphor: Color::rgba(0.84, 0.74, 1.0, 1.0),
        },
        hardware: Hardware {
            caps: CapStyle::Dome,
            cap: hex(0x0D0B10),
            pointer: gilt,
            lamp: [1.1, 0.55, 1.8],
        },
        ..base
    }
}

/// The machine window: the running machine's name and what the
/// shared dials mean on it.
fn machine(f: &Frame<'_>, r: Rect, s: &mut Scene) {
    let ty = &f.theme.typography;
    let ph = f.theme.palette.phosphor;
    let soft = Color::rgba(ph.r, ph.g, ph.b, 0.6);
    let i = step_of(f.bound(0), TOPOLOGIES.len() as u32) as usize;
    let i = i.min(LEGENDS.len() - 1);
    let x = r.x + 16.0;
    s.text_in(pt(x, r.y + 26.0), TOPOLOGIES[i], 15.0, ph, ty.legend.tracking(0.18).align(TextAlign::Left));
    for (k, line) in LEGENDS[i].iter().enumerate() {
        s.text_in(pt(x, r.y + 48.0 + 15.0 * k as f64), *line, 9.0, soft, ty.value.align(TextAlign::Left));
    }
}

pub fn face(p: &VaultParams) -> Face {
    let theme = theme();
    let gap = theme.space(3);
    let wide = theme.space(4);

    let header = row(vec![
        Title::new("VAULT").tagline("a place, not an algorithm").node(),
        spacer(),
        Presets::default().node(),
        Footswitch::bypass(p.bypass.bind()).compact().node(),
    ])
    .gap(wide)
    .align(Align::Center);

    let machine = Plate::titled("MACHINE").around(
        row(vec![
            Selector::new(p.topology.bind()).node(),
            Display::new(330.0, 106.0, machine).reads(&[p.topology.bind()]).node(),
        ])
        .gap(gap)
        .align(Align::Center),
    );
    let output = Plate::titled("OUTPUT").around(
        row(vec![
            Knob::new(p.mix.bind()).large().node(),
            Knob::new(p.duck.bind()).node(),
            Knob::new(p.output.bind()).centered().node(),
        ])
        .gap(wide)
        .align(Align::End),
    );

    let space = Plate::titled("SPACE").around(
        row(vec![
            Knob::new(p.size.bind()).large().node(),
            Knob::new(p.predelay.bind()).node(),
            Knob::new(p.width.bind()).node(),
            Knob::new(p.diffuse.bind()).node(),
        ])
        .gap(gap)
        .align(Align::End),
    );
    let energy = Plate::titled("ENERGY").around(
        row(vec![
            Knob::new(p.decay.bind()).large().node(),
            Knob::new(p.bass.bind()).label("LOW").node(),
            Knob::new(p.damp.bind()).label("HIGH").node(),
            Knob::new(p.tone.bind()).centered().node(),
        ])
        .gap(gap)
        .align(Align::End),
    );

    let room = Plate::titled("ROOM").around(
        row(vec![
            Knob::new(p.distance.bind()).node(),
            Knob::new(p.stand.bind()).centered().node(),
            Knob::new(p.early.bind()).node(),
            Selector::new(p.material.bind()).node(),
        ])
        .gap(gap)
        .align(Align::Center),
    );
    // The running machine's plate, and only it: each plate follows
    // its step of the topology selector.
    let one = |title: &'static str, b: Bind| Plate::titled(title).around(Knob::new(b).node());
    let topology = p.topology.bind();
    let machines = stack(
        [
            room,
            one("PLATE", p.stiff.bind()),
            one("MEMORY", p.ascend.bind()),
            one("DETONATE", p.fuse.bind()),
            one("VOID", p.dawn.bind()),
        ]
        .into_iter()
        .zip(0..)
        .map(|(plate, step)| on_step(plate, topology, step))
        .collect(),
    );

    let body = column(vec![
        header,
        row(vec![machine, output]).gap(gap).align(Align::Stretch),
        row(vec![space, energy]).gap(gap).align(Align::Stretch),
        machines,
    ])
    .gap(theme.space(3))
    .align(Align::Stretch);
    Face::new(theme, Panel { margin: 18.0, pad: 24.0, ..Panel::default() }.around(body))
}
