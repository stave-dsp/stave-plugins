//! The engram face: three glass windows show the machine thinking —
//! MEMORY (the bank, what it hears, how loudly it recalls), NEURONS
//! (the room's sixteen lines: grab one, drag it) and SPACE (the map of
//! learned rooms: click one) — over the room's controls (big: the room
//! and where it is on the map), the memory's (smaller), and the
//! footswitches. Output trim stays in the host's menus.

use stave::prelude::*;

use crate::params::EngramParams;
use crate::vfd;
use crate::EngramTaps;

pub fn face(p: &EngramParams) -> Face {
    let theme = Theme::indigo();
    let (s, m) = (theme.space(3), theme.space(2));
    let header = row(vec![
        Title::new("ENGRAM").tagline("a room that remembers where it heard you").node(),
        spacer(),
        Presets { width: 220.0 }.node(),
    ])
    .align(Align::Start)
    .gap(s);
    let memory = Display::new(vfd::memory::W, vfd::SCREEN_H, vfd::memory::draw).taps(EngramTaps::ALL).node();
    let nodes = [
        (p.node_0_x.bind(), p.node_0_y.bind()),
        (p.node_1_x.bind(), p.node_1_y.bind()),
        (p.node_2_x.bind(), p.node_2_y.bind()),
        (p.node_3_x.bind(), p.node_3_y.bind()),
        (p.node_4_x.bind(), p.node_4_y.bind()),
        (p.node_5_x.bind(), p.node_5_y.bind()),
        (p.node_6_x.bind(), p.node_6_y.bind()),
        (p.node_7_x.bind(), p.node_7_y.bind()),
        (p.node_8_x.bind(), p.node_8_y.bind()),
        (p.node_9_x.bind(), p.node_9_y.bind()),
        (p.node_10_x.bind(), p.node_10_y.bind()),
        (p.node_11_x.bind(), p.node_11_y.bind()),
        (p.node_12_x.bind(), p.node_12_y.bind()),
        (p.node_13_x.bind(), p.node_13_y.bind()),
        (p.node_14_x.bind(), p.node_14_y.bind()),
        (p.node_15_x.bind(), p.node_15_y.bind()),
    ];
    let screens = row(vec![memory, vfd::neurons::network(nodes), vfd::space::map(p.space_x.bind(), p.space_y.bind())])
        .align(Align::Center)
        .justify(Justify::SpaceBetween)
        .gap(s);
    let knobs = |knobs: Vec<FaceNode>| row(knobs).align(Align::Center).gap(m);
    let room = Plate::titled("ROOM").around(knobs(vec![
        Knob::new(p.size.bind()).large().node(),
        Knob::new(p.decay.bind()).large().node(),
        Knob::new(p.tone.bind()).large().node(),
    ]));
    let space = Plate::titled("SPACE").around(knobs(vec![
        Knob::new(p.space.bind()).large().node(),
        Knob::new(p.steer.bind()).large().node(),
        Knob::new(p.drift.bind()).large().node(),
    ]));
    let output = Plate::titled("OUTPUT").around(Knob::new(p.mix.bind()).large().node());
    let recall = Plate::titled("MEMORY").around(knobs(vec![
        Knob::new(p.recall.bind()).node(),
        Knob::new(p.focus.bind()).node(),
        Knob::new(p.deja.bind()).label("DÉJÀ").node(),
        Knob::new(p.matching.bind()).node(),
        Knob::new(p.texture.bind()).node(),
    ]));
    let network = Plate::titled("NEURONS").around(Knob::new(p.plasticity.bind()).label("PLASTIC").node());
    let feet = row(vec![
        Footswitch::bypass(p.bypass.bind()).node(),
        Footswitch::new(p.forget.bind()).compact().momentary().node(),
        Footswitch::new(p.hold.bind()).node(),
    ])
    .align(Align::Start)
    .gap(s);
    let body = column(vec![
        header,
        screens,
        row(vec![room, space, output]).align(Align::Stretch).gap(s),
        row(vec![recall, network, feet]).align(Align::Center).gap(s),
    ])
    .align(Align::Stretch)
    .gap(s);
    Face::new(theme, Panel::default().around(body))
}
