//! The aeolian face: the harp window across the middle, flanked by
//! the two big brass controls that play it (WIND wakes the strings,
//! SUSTAIN keeps them ringing); underneath, the strings (tone, the
//! register selector, mix), the motion (drift with its AIR and SWAY
//! buttons) and the footswitch. Output trim stays in the host's menus.

use stave::prelude::*;

use crate::harp;
use crate::params::AeolianParams;
use crate::AeolianTaps;

pub fn face(p: &AeolianParams) -> Face {
    let header = row(vec![
        Title::new("AEOLIAN").tagline("a harp the music plays").node(),
        spacer(),
        Presets { width: 210.0 }.node(),
    ])
    .align(Align::Start)
    .gap(16.0);
    let mut reads = AeolianTaps::RINGS.to_vec();
    reads.push(AeolianTaps::CLOCK);
    let harp = Display::new(440.0, 196.0, harp::draw).taps(&reads).node();
    let strings = row(vec![Knob::new(p.wind.bind()).huge().node(), harp, Knob::new(p.sustain.bind()).huge().node()])
        .align(Align::Center)
        .justify(Justify::SpaceBetween)
        .gap(24.0);
    let strings_plate = Plate::titled("STRINGS").around(
        row(vec![
            Knob::new(p.tone.bind()).node(),
            Selector::new(p.register.bind()).node(),
            Knob::new(p.mix.bind()).node(),
        ])
        .align(Align::Center)
        .gap(16.0),
    );
    // Centred in its plate, which the strings' selector makes taller.
    let motion = Plate::titled("MOTION").around(
        column(vec![row(vec![
            Knob::new(p.drift.bind()).node(),
            Footswitch::new(p.air.bind()).compact().node(),
            Footswitch::new(p.sway.bind()).compact().node(),
        ])
        .align(Align::Center)
        .gap(12.0)])
        .height(Dim::Fill(1.0))
        .justify(Justify::Center),
    );
    let controls = row(vec![
        strings_plate,
        motion,
        column(vec![Footswitch::bypass(p.bypass.bind()).node()]).justify(Justify::Center),
    ])
    .align(Align::Stretch)
    .gap(16.0);
    let body = column(vec![header, strings, controls]).align(Align::Stretch).gap(24.0);
    Face::new(Theme::verdigris(), Panel::default().around(body))
}

