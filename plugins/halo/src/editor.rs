//! The halo face — the whole window, a pure function of params. A
//! high-headroom clean head: cream tolex with dark piping, a silver
//! grille band with the gold badge, and a brushed-silver faceplate with
//! black skirted knobs. The top row is the preamp and the stack
//! (large); the four headroom controls and the master sit beneath.

use stave::face::photo::hex;
use stave::face::{CapStyle, Coat, Coverings, Finish, Hardware, Palette};
use stave::prelude::*;

use crate::params::HaloParameters;

/// Brushed-silver faceplate, black print and knobs, cream tolex,
/// silver grille cloth.
fn theme() -> Theme {
    let ink = Color::rgba(0.10, 0.10, 0.11, 1.0);
    Theme {
        name: "halo",
        finish: Finish {
            coat: Coat::Anodized,
            surface: hex(0xC6C8CA),
            plate: hex(0xB4B7BA),
            desk: hex(0x24252A),
            radius: 12.0,
        },
        palette: Palette {
            ink,
            ink_soft: Color::rgba(0.10, 0.10, 0.11, 0.62),
            accent: Color::rgba(0.70, 0.12, 0.10, 1.0),
            screen: Color::rgba(0.07, 0.06, 0.06, 1.0),
            phosphor: Color::rgba(1.0, 0.62, 0.38, 1.0),
        },
        hardware: Hardware {
            caps: CapStyle::Dome,
            cap: hex(0x101012),
            pointer: Color::rgba(0.96, 0.96, 0.96, 1.0),
            lamp: [1.9, 0.2, 0.1],
        },
        coverings: Coverings {
            tolex: hex(0xD8C79E),
            piping: hex(0x2A2018),
            grille: hex(0x8E8C88),
            badge: [0.95, 0.80, 0.48],
        },
        ..Theme::graphite()
    }
}

/// A printed section of the faceplate: its name over its knobs.
fn section(title: &'static str, knobs: Vec<FaceNode>, gap: f64) -> FaceNode {
    column(vec![
        Caption::new(title).node(),
        row(knobs).gap(gap).align(Align::End),
    ])
    .gap(6.0)
    .align(Align::Center)
}

pub fn face(p: &HaloParameters) -> Face {
    let theme = theme();
    let gap = theme.space(3);
    let wide = theme.space(5);

    let preamp = section(
        "PREAMP",
        vec![Knob::new(p.preamp_gain.bind()).large().node()],
        gap,
    );
    let tone = section(
        "TONE",
        vec![
            Knob::new(p.bass_tone.bind()).large().node(),
            Knob::new(p.midrange_tone.bind()).large().node(),
            Knob::new(p.treble_tone.bind()).large().node(),
        ],
        gap,
    );
    let top = row(vec![
        column(vec![
            Lamp::pilot().label("POWER").node(),
            Presets::default().node(),
        ])
        .gap(gap)
        .align(Align::Start),
        spacer(),
        preamp,
        tone,
    ])
    .gap(wide)
    .align(Align::End);

    let dynamics = section(
        "DYNAMICS",
        vec![
            Knob::new(p.headroom.bind()).node(),
            Knob::new(p.bloom.bind()).node(),
            Knob::new(p.reserve.bind()).node(),
            Knob::new(p.response.bind()).node(),
        ],
        gap,
    );
    let master = section(
        "MASTER",
        vec![Knob::new(p.output_level.bind()).centered().node()],
        gap,
    );
    let bottom = row(vec![
        dynamics,
        spacer(),
        master,
        Footswitch::bypass(p.bypass.bind()).compact().node(),
    ])
    .gap(wide)
    .align(Align::End);

    let body = column(vec![top, bottom])
        .gap(theme.space(4))
        .align(Align::Stretch);
    Face::new(
        theme,
        Panel::amp_head()
            .grille(Side::Top, 116.0, Some("Halo"))
            .around(body),
    )
}
