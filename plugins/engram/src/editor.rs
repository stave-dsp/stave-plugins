//! The engram face: three glass windows show the machine thinking —
//! MEMORY (the bank, what it hears, how loudly it recalls), NEURONS
//! (the room's sixteen lines: grab one, drag it) and SPACE (the map of
//! learned rooms: click one) — over the room's controls (big: the room
//! and where it is on the map), the memory's (smaller), and the
//! footswitches. Output trim stays in the host's menus.

use stave_core::face::prelude::*;
use stave_core::params::BindParam;

use crate::params::EngramParams;
use crate::vfd;

pub fn face(p: &EngramParams) -> Face {
    let header = ui! {
        <Row align=Start gap=16>
            {Title::new("ENGRAM").tagline("a room that remembers where it heard you").node()}
            <Spacer/>
            {Presets { width: 220.0 }.node()}
        </Row>
    };
    let memory = Display::new(vfd::memory::W, vfd::SCREEN_H, vfd::memory::draw).probes(&crate::FACE_TAPS).node();
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
    let screens = ui! {
        <Row align=Center justify=SpaceBetween gap=16>
            {memory}
            {vfd::neurons::network(nodes)}
            {vfd::space::map(p.space_x.bind(), p.space_y.bind())}
        </Row>
    };
    let room = Plate::titled("ROOM").around(ui! {
        <Row align=Center gap=12>
            {Knob::new(p.size.bind()).large().node()}
            {Knob::new(p.decay.bind()).large().node()}
            {Knob::new(p.tone.bind()).large().node()}
        </Row>
    });
    let space = Plate::titled("SPACE").around(ui! {
        <Row align=Center gap=12>
            {Knob::new(p.space.bind()).large().node()}
            {Knob::new(p.steer.bind()).large().node()}
            {Knob::new(p.drift.bind()).large().node()}
        </Row>
    });
    let output = Plate::titled("OUTPUT").around(Knob::new(p.mix.bind()).large().node());
    let recall = Plate::titled("MEMORY").around(ui! {
        <Row align=Center gap=12>
            {Knob::new(p.recall.bind()).node()}
            {Knob::new(p.focus.bind()).node()}
            {Knob::new(p.deja.bind()).label("DÉJÀ").node()}
            {Knob::new(p.matching.bind()).node()}
            {Knob::new(p.texture.bind()).node()}
        </Row>
    });
    let network = Plate::titled("NEURONS").around(Knob::new(p.plasticity.bind()).label("PLASTIC").node());
    let feet = ui! {
        <Row align=Start gap=16>
            {Footswitch::bypass(p.bypass.bind()).node()}
            {Footswitch::new(p.forget.bind()).compact().momentary().node()}
            {Footswitch::new(p.hold.bind()).node()}
        </Row>
    };
    let body = ui! {
        <Column align=Stretch gap=16>
            {header}
            {screens}
            <Row align=Stretch gap=16>
                {room}
                {space}
                {output}
            </Row>
            <Row align=Center gap=16>
                {recall}
                {network}
                {feet}
            </Row>
        </Column>
    };
    Face::new(Theme::indigo(), Panel::default().around(body))
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_face_fits_a_laptop_screen() {
        let rt = stave_core::face::Runtime::new(super::face(&crate::params::EngramParams::new()));
        let (w, h) = rt.size();
        eprintln!("engram face: {w} × {h} pt");
        assert!(w <= 1000.0 && h <= 720.0, "engram face is {w} × {h} pt");
    }
}
