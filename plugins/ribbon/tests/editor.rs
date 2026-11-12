//! The face, as a player uses it: the tape machine's own controls do
//! what their legends say (the generic face laws run in `checks.rs`).

use stave_ribbon_vst3::Ribbon;
use stave_test::face::{dy, FaceDriver, Key};

#[test]
fn dragging_erase_down_keeps_the_loop() {
    let mut ui = FaceDriver::of::<Ribbon>();
    ui.press("erase").move_to(dy(55.0));
    assert!((ui.value("erase") - 0.75).abs() < 1e-9, "a quarter of the travel down: {}", ui.value("erase"));
    ui.release().exit();
    ui.assert_bracketed();
}

#[test]
fn the_keyboard_tops_out_intensity() {
    let mut ui = FaceDriver::of::<Ribbon>();
    ui.click("intensity");
    ui.key(Key::End);
    assert_eq!(ui.value("intensity"), 1.0);
    ui.assert_bracketed();
}

#[test]
fn the_brake_latches() {
    let mut ui = FaceDriver::of::<Ribbon>();
    ui.click("brake");
    assert_eq!(ui.value("brake"), 1.0, "the tape stops");
    ui.click("brake");
    assert_eq!(ui.value("brake"), 0.0, "and runs again");
}
