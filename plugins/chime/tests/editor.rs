//! The face, as a player uses it: Chime's own controls do what their
//! legends say (the generic face laws run in `checks.rs`).

use stave_chime_vst3::Chime;
use stave_test::face::{dy, FaceDriver, Key};

#[test]
fn dragging_sustain_up_holds_the_note_higher() {
    let mut ui = FaceDriver::of::<Chime>();
    ui.press("sustain").move_to(dy(-55.0));
    assert!((ui.value("sustain") - 0.25).abs() < 1e-9, "a quarter of the travel: {}", ui.value("sustain"));
    ui.release().exit();
    ui.assert_bracketed();
}

#[test]
fn the_keyboard_tops_out_the_output() {
    let mut ui = FaceDriver::of::<Chime>();
    ui.click("output");
    ui.key(Key::End);
    assert_eq!(ui.value("output"), 24.0);
    ui.assert_bracketed();
}

#[test]
fn the_bypass_footswitch_latches() {
    let mut ui = FaceDriver::of::<Chime>();
    ui.click("bypass");
    assert_eq!(ui.value("bypass"), 1.0);
    ui.click("bypass");
    assert_eq!(ui.value("bypass"), 0.0);
}

/// Every page fits the window budget.
#[test]
fn the_face_fits() {
    let (w, h) = stave_test::FaceDriver::of::<stave_chime_vst3::Chime>().size();
    assert!(w <= 1000.0 && h <= 720.0, "chime face is {w}×{h}");
}
