//! The face, as a player uses it: Halo's own controls do what their
//! legends say (the generic face laws run in `checks.rs`).

use stave_halo_vst3::Halo;
use stave_test::face::{dy, FaceDriver, Key};

#[test]
fn dragging_gain_up_raises_it_and_shows_the_value() {
    let mut ui = FaceDriver::of::<Halo>();
    ui.press("gain").move_to(dy(-55.0));
    let gain = ui.value("gain");
    assert!(
        (gain - 10.5).abs() < 1e-9,
        "a quarter of the travel is a quarter of the range: {gain}"
    );
    assert!(
        ui.shows("10.5 dB"),
        "the readout shows the value while dragging: {:?}",
        ui.texts()
    );
    ui.release().exit();
    ui.assert_bracketed();
}

#[test]
fn the_keyboard_sends_a_tone_knob_to_its_ends() {
    let mut ui = FaceDriver::of::<Halo>();
    ui.click("treble");
    ui.key(Key::End);
    assert_eq!(ui.value("treble"), 10.0);
    ui.key(Key::Home);
    assert_eq!(ui.value("treble"), 0.0);
    ui.assert_bracketed();
}

#[test]
fn the_bypass_footswitch_latches() {
    let mut ui = FaceDriver::of::<Halo>();
    ui.click("bypass");
    assert_eq!(ui.value("bypass"), 1.0);
    ui.click("bypass");
    assert_eq!(ui.value("bypass"), 0.0);
}
