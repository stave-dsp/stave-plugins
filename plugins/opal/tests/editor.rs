//! The face, as a person uses it: Opal's signature controls do what
//! their legends say (the generic face laws run in `checks.rs`).

use stave_opal_vst3::Opal;
use stave_test::face::{dy, FaceDriver, Key};

#[test]
fn dragging_feedback_up_grows_the_teeth() {
    let mut ui = FaceDriver::of::<Opal>();
    assert_eq!(ui.value("feedback"), 0.0);
    ui.press("feedback").move_to(dy(-55.0));
    assert!(ui.value("feedback") > 0.2, "feedback {}", ui.value("feedback"));
    ui.release();
    ui.assert_bracketed();
}

#[test]
fn the_keyboard_steps_the_algorithm() {
    let mut ui = FaceDriver::of::<Opal>();
    ui.click("algo");
    ui.key(Key::End);
    assert_eq!(ui.value("algo"), 7.0, "End lands on ORGAN");
    ui.key(Key::Home);
    assert_eq!(ui.value("algo"), 0.0, "Home lands on CHAIN");
    ui.assert_bracketed();
}

#[test]
fn the_bypass_footswitch_latches() {
    let mut ui = FaceDriver::of::<Opal>();
    ui.click("bypass");
    assert_eq!(ui.value("bypass"), 1.0);
    ui.click("bypass");
    assert_eq!(ui.value("bypass"), 0.0);
}

/// Every page fits the window budget.
#[test]
fn the_face_fits() {
    let (w, h) = stave_test::FaceDriver::of::<stave_opal_vst3::Opal>().size();
    assert!(w <= 1000.0 && h <= 720.0, "opal face is {w}×{h}");
}
