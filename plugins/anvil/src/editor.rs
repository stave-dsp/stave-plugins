//! The anvil face — the whole window, a pure function of params. A
//! modern high-gain head: black tolex with red piping, a black grille
//! band carrying the chrome badge, and a black crinkle faceplate with
//! ivory chicken-heads. The faceplate reads like the amp's signal
//! path: the preamp and the stack on top (large), the
//! voicing, the power section and the master beneath.

use stave::face::photo::hex;
use stave::face::{CapStyle, Coat, Coverings, Finish, Hardware, Palette};
use stave::prelude::*;

use crate::params::AnvilParameters;

/// Black crinkle faceplate, chrome print, ivory chicken-heads, black
/// tolex with red piping.
fn theme() -> Theme {
    let base = Theme::graphite();
    Theme {
        name: "anvil",
        finish: Finish {
            coat: Coat::Crinkle,
            surface: hex(0x1A1A1C),
            plate: hex(0x121213),
            desk: hex(0x0B0B0C),
            radius: 10.0,
        },
        palette: Palette {
            ink: Color::rgba(0.93, 0.93, 0.94, 1.0),
            ink_soft: Color::rgba(0.93, 0.93, 0.94, 0.55),
            accent: Color::rgba(1.0, 0.24, 0.14, 1.0),
            screen: Color::rgba(0.03, 0.03, 0.03, 1.0),
            phosphor: Color::rgba(1.0, 0.4, 0.22, 1.0),
        },
        hardware: Hardware {
            caps: CapStyle::Chicken,
            cap: hex(0xE6E2D8),
            pointer: Color::rgba(0.07, 0.07, 0.08, 1.0),
            lamp: [1.95, 0.22, 0.08],
        },
        coverings: Coverings {
            tolex: hex(0x0F0F10),
            piping: hex(0xA3161A),
            grille: hex(0x1C1C1E),
            badge: [0.9, 0.9, 0.93],
        },
        ..base
    }
}

/// A printed section of the faceplate: its name over its knobs.
fn section(title: &'static str, knobs: Vec<FaceNode>, gap: f64) -> FaceNode {
    column(vec![Caption::new(title).node(), row(knobs).gap(gap).align(Align::End)])
        .gap(6.0)
        .align(Align::Center)
}

pub fn face(p: &AnvilParameters) -> Face {
    let theme = theme();
    let gap = theme.space(3);
    let wide = theme.space(5);

    let preamp = section(
        "PREAMP",
        vec![
            Knob::new(p.preamp_gain.bind()).large().node(),
            Knob::new(p.topology_separation_amount.bind()).large().node(),
        ],
        gap,
    );
    let tone = section(
        "TONE STACK",
        vec![
            Knob::new(p.bass_tone.bind()).large().node(),
            Knob::new(p.midrange_tone.bind()).large().node(),
            Knob::new(p.treble_tone.bind()).large().node(),
        ],
        gap,
    );
    let top = row(vec![
        column(vec![Lamp::pilot().label("POWER").node(), Presets::default().node()])
            .gap(gap)
            .align(Align::Start),
        spacer(),
        preamp,
        tone,
    ])
    .gap(wide)
    .align(Align::End);

    let voicing = section(
        "VOICING",
        vec![
            Knob::new(p.input_low_cut.bind()).node(),
            Knob::new(p.difference_tone.bind()).node(),
            Knob::new(p.high_frequency_emphasis.bind()).node(),
        ],
        gap,
    );
    let power = section(
        "POWER",
        vec![
            Knob::new(p.power_pair_overdrive.bind()).node(),
            Knob::new(p.presence_tone.bind()).centered().node(),
            Knob::new(p.depth_tone.bind()).centered().node(),
        ],
        gap,
    );
    let master = section("MASTER", vec![Knob::new(p.output_level.bind()).centered().node()], gap);
    let bottom = row(vec![
        voicing,
        power,
        master,
        spacer(),
        Footswitch::bypass(p.bypass.bind()).compact().node(),
    ])
    .gap(wide)
    .align(Align::End);

    let body = column(vec![top, bottom]).gap(theme.space(4)).align(Align::Stretch);
    Face::new(theme, Panel::amp_head().grille(Side::Top, 116.0, Some("ANVIL")).around(body))
}
