//! The face, as a person uses it: Tolex's own controls do what their
//! legends say (the generic face laws run in `checks.rs`).

use stave_test::face::{dy, FaceDriver, Key};
use stave_tolex_vst3::Tolex;

/// The face fits a 1000 × 720 pt window.
#[test]
fn face_fits_the_window() {
    let (w, h) = FaceDriver::of::<Tolex>().size();
    assert!(w <= 1000.0 && h <= 720.0, "face is {w} x {h}");
}

#[test]
fn dragging_the_drive_knob_up_swings_the_cone_harder_and_shows_it() {
    let mut ui = FaceDriver::of::<Tolex>();
    assert_eq!(ui.value("drive"), 6.0);
    ui.press("drive").move_to(dy(-55.0));
    assert_eq!(
        ui.value("drive"),
        15.0,
        "a quarter of the travel is a quarter of the range"
    );
    assert!(
        ui.shows("15.0 dB"),
        "the readout shows the value while dragging: {:?}",
        ui.texts()
    );
    ui.release().exit();
    ui.assert_bracketed();
}

#[test]
fn the_back_toggle_opens_the_cabinet() {
    let mut ui = FaceDriver::of::<Tolex>();
    assert_eq!(ui.value("back"), 0.0, "BREAKDOWN is closed-back");
    ui.click("back");
    assert_eq!(ui.value("back"), 1.0);
    assert!(ui.shows("OPEN"), "{:?}", ui.texts());
}

#[test]
fn the_keyboard_walks_the_mic_to_the_end_of_its_range() {
    let mut ui = FaceDriver::of::<Tolex>();
    ui.click("mic");
    assert_eq!(ui.focus(), Some(ui.part("mic")));
    ui.key(Key::End);
    assert_eq!(ui.value("mic"), 60.0);
    ui.key(Key::Home);
    assert_eq!(ui.value("mic"), 0.5);
    ui.assert_bracketed();
}
