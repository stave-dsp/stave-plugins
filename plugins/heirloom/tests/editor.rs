//! The face, as a person uses it: Heirloom's signature controls do
//! what their legends say (the generic face laws run in `checks.rs`).

use stave_heirloom_vst3::Heirloom;
use stave_test::face::{dy, FaceDriver};

#[test]
fn dragging_repeats_up_a_quarter_adds_a_quarter() {
    let mut ui = FaceDriver::of::<Heirloom>();
    ui.press("repeats").move_to(dy(-55.0));
    assert!((ui.value("repeats") - 0.75).abs() < 1e-6, "repeats {}", ui.value("repeats"));
    ui.release().exit();
    ui.assert_bracketed();
}

#[test]
fn the_machine_toggle_throws_to_the_next_machine() {
    let mut ui = FaceDriver::of::<Heirloom>();
    assert_eq!(ui.value("machine"), 0.0);
    ui.click("machine");
    assert_ne!(ui.value("machine"), 0.0, "throwing the toggle changes the machine");
    ui.assert_bracketed();
}

#[test]
fn the_hold_footswitch_latches_and_releases() {
    let mut ui = FaceDriver::of::<Heirloom>();
    ui.click("hold");
    assert_eq!(ui.value("hold"), 1.0);
    ui.click("hold");
    assert_eq!(ui.value("hold"), 0.0);
}
