//! The Forge face — the whole window, a pure function of params: a
//! bronze hammertone amp head. The operating point sits up top (the
//! bias × drive pad, the live transfer, the harmonic ladder and the
//! stage strip between the I/O meters); the voicing reads below in
//! signal order: STAGE 1 → TOPOLOGY, then TEXTURE → SUB → OUTPUT.

use stave::face::photo::hex;
use stave::face::{CapStyle, Coat, Finish, Hardware, Palette};
use stave::prelude::*;

use crate::engine::ForgeTaps;
use crate::params::ForgeParams;
use crate::views;

/// Forge's look: warm bronze hammertone, cream print, ember light.
pub fn theme() -> Theme {
    let base = Theme::graphite();
    Theme {
        name: "forge",
        finish: Finish {
            coat: Coat::Hammertone,
            surface: hex(0x5A3A22),
            plate: hex(0x452B19),
            desk: hex(0x1C1A18),
            ..base.finish
        },
        palette: Palette {
            ink: Color::rgba(0.95, 0.89, 0.78, 1.0),
            ink_soft: Color::rgba(0.95, 0.89, 0.78, 0.58),
            accent: Color::rgba(1.0, 0.56, 0.22, 1.0),
            screen: Color::rgba(0.055, 0.04, 0.03, 1.0),
            phosphor: Color::rgba(1.0, 0.64, 0.30, 1.0),
        },
        hardware: Hardware {
            caps: CapStyle::Dome,
            cap: hex(0x141210),
            pointer: Color::rgba(0.98, 0.93, 0.84, 1.0),
            lamp: [1.8, 0.5, 0.1],
        },
        ..base
    }
}

/// Meter + trim knob: one side of the I/O.
fn io(tap: TapId, label: &'static str, trim: Bind) -> FaceNode {
    column(vec![
        Meter::db(tap).label(label).height(110.0).node(),
        Knob::new(trim).small().centered().node(),
    ])
    .gap(8.0)
    .align(Align::Center)
}

pub fn face(p: &ForgeParams) -> Face {
    let theme = theme();
    let gap = theme.space(2);
    let row_gap = theme.space(3);

    let header = row(vec![
        Title::new("FORGE")
            .tagline("amp-topology saturation")
            .node(),
        spacer(),
        Presets { width: 220.0 }.node(),
    ])
    .gap(theme.space(4))
    .align(Align::Center);

    const SCREEN_W: f64 = 184.0;
    const SCREEN_H: f64 = 104.0;
    let screens = row(vec![
        Pad::new(
            p.bias.bind(),
            p.drive.bind(),
            SCREEN_W + 4.0,
            SCREEN_H + 4.0,
        )
        .node(),
        Display::new(SCREEN_W, SCREEN_H, views::transfer).node(),
        Display::new(SCREEN_W, SCREEN_H, views::harmonics)
            .taps(&[ForgeTaps::VIZ_AMP])
            .node(),
    ])
    .gap(gap)
    .align(Align::Center);
    let strip = Display::new(
        3.0 * (SCREEN_W + 10.0) + 2.0 * gap - 10.0,
        30.0,
        views::topology,
    )
    .taps(&ForgeTaps::STAGES)
    .reads(&[
        p.character.bind(),
        p.stack.bind(),
        p.filter.bind(),
        p.post.bind(),
    ])
    .node();
    let engine = Plate::titled("OPERATING POINT")
        .around(column(vec![screens, strip]).gap(gap).align(Align::Center));
    let top = row(vec![
        io(ForgeTaps::IO_IN, "IN", p.input.bind()),
        engine,
        io(ForgeTaps::IO_OUT, "OUT", p.output.bind()),
    ])
    .gap(row_gap)
    .align(Align::Center);

    let stage1 = Plate::titled("STAGE 1").around(
        row(vec![
            Knob::new(p.drive.bind()).node(),
            Selector::new(p.character.bind()).node(),
            Knob::new(p.edge.bind()).node(),
            Knob::new(p.bias.bind()).node(),
        ])
        .gap(gap)
        .align(Align::Center),
    );
    let topology = Plate::titled("TOPOLOGY").around(
        row(vec![
            Selector::new(p.stack.bind()).node(),
            column(vec![
                Stepper::new(p.filter.bind()).width(124.0).node(),
                Stepper::new(p.post.bind())
                    .width(124.0)
                    .label("POST")
                    .node(),
            ])
            .gap(gap),
            Knob::new(p.tone.bind()).centered().node(),
        ])
        .gap(gap)
        .align(Align::Center),
    );
    let texture = Plate::titled("TEXTURE").around(
        row(vec![
            Selector::new(p.memory.bind()).node(),
            Knob::new(p.depth.bind()).node(),
            Knob::new(p.grit.bind()).node(),
            Knob::new(p.iron.bind()).node(),
        ])
        .gap(gap)
        .align(Align::Center),
    );
    let sub = Plate::titled("SUB").around(
        row(vec![
            Knob::new(p.sub_mix.bind()).node(),
            Knob::new(p.sub_mod.bind()).node(),
        ])
        .gap(gap)
        .align(Align::Center),
    );
    let output = Plate::titled("OUTPUT").around(
        row(vec![
            Knob::new(p.width.bind()).node(),
            Knob::new(p.mix.bind()).node(),
            Footswitch::bypass(p.bypass.bind()).compact().node(),
        ])
        .gap(gap)
        .align(Align::Center),
    );

    let body = column(vec![
        header,
        top,
        row(vec![stage1, topology])
            .gap(row_gap)
            .align(Align::Stretch),
        row(vec![texture, sub, output])
            .gap(row_gap)
            .align(Align::Stretch),
    ])
    .gap(row_gap)
    .align(Align::Stretch);
    let panel = Panel {
        margin: 16.0,
        pad: 20.0,
        screws: true,
        ..Panel::default()
    };
    Face::new(theme, panel.around(body))
}
