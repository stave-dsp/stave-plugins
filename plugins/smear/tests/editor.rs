//! The face, as a player uses it: the garden's own controls do what
//! their legends say (the generic face laws run in `checks.rs`).

use stave_smear_vst3::Smear;
use stave_test::face::{dy, FaceDriver, Key};

#[test]
fn dragging_seed_up_drives_the_garden_harder() {
    let mut ui = FaceDriver::of::<Smear>();
    ui.press("seed").move_to(dy(-55.0));
    assert!((ui.value("seed") - 0.75).abs() < 1e-9, "a quarter of the travel: {}", ui.value("seed"));
    ui.release().exit();
    ui.assert_bracketed();
}

#[test]
fn the_keyboard_tops_out_the_output() {
    let mut ui = FaceDriver::of::<Smear>();
    ui.click("output");
    ui.key(Key::End);
    assert_eq!(ui.value("output"), 12.0);
    ui.assert_bracketed();
}

/// The face fits a 1000 × 720 pt window.
#[test]
fn face_fits_the_window() {
    let (w, h) = FaceDriver::of::<Smear>().size();
    assert!(w <= 1000.0 && h <= 720.0, "face is {w} x {h}");
}
