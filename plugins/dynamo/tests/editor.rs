//! The face, as a person uses it: Dynamo's own controls do what their
//! legends say (the generic face laws run in `checks.rs`).

use stave_dynamo_vst3::Dynamo;
use stave_test::face::{dy, FaceDriver, Key};

/// The MACHINE page's macro knob: a quarter of the travel up is a
/// quarter of the range, shown while dragging.
#[test]
fn dragging_the_force_macro_drives_the_machine_harder() {
    let mut ui = FaceDriver::of::<Dynamo>();
    ui.press("force").move_to(dy(-55.0));
    assert_eq!(ui.value("force"), 0.75, "a quarter of the travel is a quarter of the range");
    ui.release().exit();
    ui.assert_bracketed();
}

/// Each macro's MOTION source sits under its knob: the keyboard walks
/// it to the end of the source list (SYMP).
#[test]
fn a_macros_motion_source_steps_from_the_keyboard() {
    let mut ui = FaceDriver::of::<Dynamo>();
    assert_eq!(ui.value("src 5"), 0.0, "motion starts OFF");
    ui.click("src 5");
    assert_eq!(ui.focus(), Some(ui.part("src 5")));
    ui.key(Key::End);
    assert_eq!(ui.value_text("src 5"), "SYMP");
    ui.assert_bracketed();
}

#[test]
fn the_bypass_footswitch_latches() {
    let mut ui = FaceDriver::of::<Dynamo>();
    ui.click("bypass");
    assert_eq!(ui.value("bypass"), 1.0);
    ui.click("bypass");
    assert_eq!(ui.value("bypass"), 0.0);
}

/// Every page fits the window budget.
#[test]
fn the_face_fits() {
    let (w, h) = stave_test::FaceDriver::of::<stave_dynamo_vst3::Dynamo>().size();
    assert!(w <= 1000.0 && h <= 720.0, "dynamo face is {w}×{h}");
}
