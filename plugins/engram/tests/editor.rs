//! The face, as a person uses it: Engram's own controls do what their
//! legends say (the generic face laws run in `checks.rs`; the NEURONS
//! drag in `neuron_drag.rs`).

use stave_test::face::{dy, FaceDriver};
use stave_engram_vst3::Engram;

/// The SPACE map draws y UP: a click puts the cursor (SPACE X / Y) at
/// the point under the pointer.
#[test]
#[cfg_attr(miri, ignore)]
fn clicking_the_map_puts_the_cursor_under_the_pointer() {
    let mut ui = FaceDriver::of::<Engram>();
    let map = match ui.grabs("space x").first().map(|g| g.gesture.clone()) {
        Some(stave::face::Gesture::Pad { area, .. }) => area,
        other => panic!("the SPACE map is a pad: {other:?}"),
    };
    // A quarter across, near the top.
    ui.click((map.x + 0.25 * map.w, map.y + 0.2 * map.h));
    assert!((ui.normalized("space x") - 0.25).abs() < 1e-9, "x {}", ui.normalized("space x"));
    assert!((ui.normalized("space y") - 0.8).abs() < 1e-9, "near the top is high y: {}", ui.normalized("space y"));
    ui.assert_bracketed();
}

/// FORGET is momentary (any press wipes the bank); HOLD latches.
#[test]
#[cfg_attr(miri, ignore)]
fn forget_springs_back_and_hold_latches() {
    let mut ui = FaceDriver::of::<Engram>();
    ui.press("forget");
    assert_eq!(ui.value("forget"), 1.0, "down while held");
    ui.release();
    assert_eq!(ui.value("forget"), 0.0, "and springs back");
    ui.click("hold");
    assert_eq!(ui.value("hold"), 1.0);
    ui.click("hold");
    assert_eq!(ui.value("hold"), 0.0);
    ui.assert_bracketed();
}

/// DECAY's knob sweeps its log range: a full drag up is 30 s, and the
/// readout shows it while dragging.
#[test]
#[cfg_attr(miri, ignore)]
fn dragging_decay_all_the_way_up_is_thirty_seconds() {
    let mut ui = FaceDriver::of::<Engram>();
    ui.press("decay").move_to(dy(-1000.0));
    assert_eq!(ui.value("decay"), 30.0);
    assert!(ui.shows("30.0"), "the readout shows the value while dragging: {:?}", ui.texts());
    ui.release().exit();
    ui.assert_bracketed();
}

/// Every page fits the window budget.
#[test]
fn the_face_fits() {
    let (w, h) = stave_test::FaceDriver::of::<stave_engram_vst3::Engram>().size();
    assert!(w <= 1000.0 && h <= 720.0, "engram face is {w}×{h}");
}
