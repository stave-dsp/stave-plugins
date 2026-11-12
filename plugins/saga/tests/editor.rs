//! The face, as a person uses it: Saga's signature controls do what
//! their legends say (the generic face laws run in `checks.rs`).

use stave_saga_vst3::Saga;
use stave_test::face::{dy, FaceDriver};

#[test]
fn dragging_feedback_up_a_quarter_adds_a_quarter() {
    let mut ui = FaceDriver::of::<Saga>();
    ui.press("feedback").move_to(dy(-55.0));
    assert!((ui.value("feedback") - 0.70).abs() < 1e-6, "feedback {}", ui.value("feedback"));
    ui.release().exit();
    ui.assert_bracketed();
}

#[test]
fn the_mode_selector_steps_through_the_programs() {
    let mut ui = FaceDriver::of::<Saga>();
    assert_eq!(ui.value("mode"), 0.0);
    ui.click("mode");
    assert_ne!(ui.value("mode"), 0.0, "clicking the selector changes the program");
    ui.assert_bracketed();
}

#[test]
fn the_dive_footswitch_latches_and_releases() {
    let mut ui = FaceDriver::of::<Saga>();
    ui.click("dive");
    assert_eq!(ui.value("dive"), 1.0);
    ui.click("dive");
    assert_eq!(ui.value("dive"), 0.0);
}
