//! The aeolian face: the harp window across the middle, flanked by
//! the two big brass controls that play it (WIND wakes the strings,
//! SUSTAIN keeps them ringing); underneath, the strings (tone, the
//! register selector, mix), the motion (drift with its AIR and SWAY
//! buttons) and the footswitch. Output trim stays in the host's menus.

use stave_core::face::prelude::*;
use stave_core::params::BindParam;

use crate::harp;
use crate::params::AeolianParams;

pub fn face(p: &AeolianParams) -> Face {
    let header = ui! {
        <Row align=Start gap=16>
            {Title::new("AEOLIAN").tagline("a harp the music plays").node()}
            <Spacer/>
            {Presets { width: 210.0 }.node()}
        </Row>
    };
    let harp = Display::new(440.0, 196.0, harp::draw).probes(&crate::FACE_TAPS).node();
    let strings = ui! {
        <Row align=Center justify=SpaceBetween gap=24>
            {Knob::new(p.wind.bind()).huge().node()}
            {harp}
            {Knob::new(p.sustain.bind()).huge().node()}
        </Row>
    };
    let strings_plate = Plate::titled("STRINGS").around(ui! {
        <Row align=Center gap=16>
            {Knob::new(p.tone.bind()).node()}
            {Selector::new(p.register.bind()).node()}
            {Knob::new(p.mix.bind()).node()}
        </Row>
    });
    // Centred in its plate, which the strings' selector makes taller.
    let motion = Plate::titled("MOTION").around(ui! {
        <Column h=Fill justify=Center>
            <Row align=Center gap=12>
                {Knob::new(p.drift.bind()).node()}
                {Footswitch::new(p.air.bind()).compact().node()}
                {Footswitch::new(p.sway.bind()).compact().node()}
            </Row>
        </Column>
    });
    let controls = ui! {
        <Row align=Stretch gap=16>
            {strings_plate}
            {motion}
            <Column justify=Center>
                {Footswitch::bypass(p.bypass.bind()).node()}
            </Column>
        </Row>
    };
    let body = ui! {
        <Column align=Stretch gap=24>
            {header}
            {strings}
            {controls}
        </Column>
    };
    Face::new(Theme::verdigris(), Panel::default().around(body))
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_face_fits_a_laptop_screen() {
        let rt = stave_core::face::Runtime::new(super::face(&crate::params::AeolianParams::new()));
        let (w, h) = rt.size();
        eprintln!("aeolian face: {w} × {h} pt");
        assert!(w <= 1000.0 && h <= 720.0, "aeolian face is {w} × {h} pt");
    }
}
