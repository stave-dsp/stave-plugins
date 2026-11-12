//! The face, as a player uses it: the harp's own controls do what their
//! legends say (the generic face laws run in `checks.rs`).

use stave_aeolian_vst3::Aeolian;
use stave_test::face::{dy, FaceDriver};

#[test]
fn dragging_wind_up_wakes_the_strings_more_easily() {
    let mut ui = FaceDriver::of::<Aeolian>();
    ui.press("wind").move_to(dy(-55.0));
    assert!((ui.value("wind") - 0.75).abs() < 1e-9, "a quarter of the travel: {}", ui.value("wind"));
    ui.release().exit();
    ui.assert_bracketed();
}

#[test]
fn air_and_sway_latch() {
    let mut ui = FaceDriver::of::<Aeolian>();
    ui.click("air");
    assert_eq!(ui.value("air"), 0.0, "AIR starts on; a click turns it off");
    ui.click("sway");
    assert_eq!(ui.value("sway"), 1.0);
    ui.click("sway");
    assert_eq!(ui.value("sway"), 0.0);
}

#[test]
fn the_face_fits_a_laptop_screen() {
    let (w, h) = FaceDriver::of::<Aeolian>().size();
    assert!(w <= 1000.0 && h <= 720.0, "aeolian face is {w} × {h} pt");
}
