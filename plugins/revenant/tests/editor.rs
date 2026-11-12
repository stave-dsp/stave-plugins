//! The face, as a player uses it: the lab instrument's own controls do
//! what their legends say (the generic face laws run in `checks.rs`).

use stave_revenant_vst3::Revenant;
use stave_test::face::{dy, FaceDriver, Key};

#[test]
fn dragging_chaos_up_winds_the_network_up() {
    let mut ui = FaceDriver::of::<Revenant>();
    ui.press("chaos").move_to(dy(-55.0));
    assert!((ui.value("chaos") - 0.70).abs() < 1e-9, "a quarter of the travel: {}", ui.value("chaos"));
    ui.release().exit();
    ui.assert_bracketed();
}

#[test]
fn the_keyboard_tops_out_dream() {
    let mut ui = FaceDriver::of::<Revenant>();
    ui.click("dream");
    ui.key(Key::End);
    assert_eq!(ui.value("dream"), 1.0);
    ui.assert_bracketed();
}

#[test]
fn sleep_latches() {
    let mut ui = FaceDriver::of::<Revenant>();
    ui.click("sleep");
    assert_eq!(ui.value("sleep"), 1.0, "it falls asleep");
    ui.click("sleep");
    assert_eq!(ui.value("sleep"), 0.0, "and wakes");
}
