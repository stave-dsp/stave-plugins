//! The face, as a player uses it: Anvil's own controls do what their
//! legends say (bindings, grabs and the generic face laws run in
//! `checks.rs`).

use stave_anvil_vst3::Anvil;
use stave_test::face::{dy, FaceDriver, Key};

#[test]
fn dragging_the_gain_knob_up_adds_gain() {
    let mut ui = FaceDriver::of::<Anvil>();
    let before = ui.value("gain");
    ui.press("gain").move_to(dy(-40.0));
    assert!(ui.value("gain") > before, "dragging up must add gain");
    ui.release().exit();
    ui.assert_bracketed();
}

#[test]
fn the_keyboard_sends_the_master_to_its_ends() {
    let mut ui = FaceDriver::of::<Anvil>();
    ui.click("output");
    ui.key(Key::End);
    assert_eq!(ui.value("output"), 24.0);
    ui.key(Key::Home);
    assert_eq!(ui.value("output"), -24.0);
    ui.assert_bracketed();
}

#[test]
fn the_bypass_footswitch_latches() {
    let mut ui = FaceDriver::of::<Anvil>();
    ui.click("bypass");
    assert_eq!(ui.value("bypass"), 1.0);
    ui.click("bypass");
    assert_eq!(ui.value("bypass"), 0.0);
}
