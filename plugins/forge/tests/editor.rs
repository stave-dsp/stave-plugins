//! The face, as a person uses Forge (the generic face laws run in
//! `checks.rs`).

use stave_forge_vst3::Forge;
use stave_test::face::{dy, FaceDriver, Key};

/// The face fits a 1000 × 720 pt window.
#[test]
fn face_fits_the_window() {
    let (w, h) = FaceDriver::of::<Forge>().size();
    assert!(w <= 1000.0 && h <= 720.0, "face is {w} x {h}");
}

#[test]
fn dragging_drive_up_heats_the_stage() {
    let mut ui = FaceDriver::of::<Forge>();
    assert_eq!(ui.value("drive"), 4.0);
    ui.press("drive").move_to(dy(-55.0));
    assert_eq!(ui.value("drive"), 6.5, "a quarter of the travel is a quarter of the range");
    ui.release().exit();
    ui.assert_bracketed();
}

#[test]
fn the_keyboard_nudges_the_focused_output_trim() {
    let mut ui = FaceDriver::of::<Forge>();
    ui.click("output");
    ui.key(Key::End);
    assert_eq!(ui.value("output"), 24.0);
    ui.key(Key::Home);
    assert_eq!(ui.value("output"), -24.0);
    ui.assert_bracketed();
}

#[test]
fn the_bypass_footswitch_latches() {
    let mut ui = FaceDriver::of::<Forge>();
    ui.click("bypass");
    assert_eq!(ui.value("bypass"), 1.0);
    ui.click("bypass");
    assert_eq!(ui.value("bypass"), 0.0);
}
