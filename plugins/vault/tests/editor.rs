//! The face, as a person uses it: the bottom row shows the running
//! machine's own plate and no other (the generic face laws run in
//! `checks.rs`).

use stave_test::face::{dy, FaceDriver, Key};
use stave_vault_vst3::Vault;

/// Each machine's own controls, by topology step: ROOM's plate, then
/// PLATE, MEMORY, DETONATE and VOID's one dial each.
const PLATES: [&[&str]; 5] = [&["distance", "stand", "early", "material"], &["whip"], &["ascend"], &["fuse"], &["dawn"]];

#[test]
fn selecting_a_machine_shows_its_plate_and_hides_the_others() {
    let mut ui = FaceDriver::of::<Vault>();
    for machine in 0..PLATES.len() {
        ui.set("topology", machine as f64);
        for (step, controls) in PLATES.iter().enumerate() {
            for control in *controls {
                let shown = !ui.grabs(*control).is_empty();
                assert_eq!(shown, step == machine, "machine {machine}: {control} shown {shown}");
            }
        }
    }
}

#[test]
fn the_machine_window_names_the_running_machine() {
    let mut ui = FaceDriver::of::<Vault>();
    ui.set("topology", 3.0);
    assert!(ui.shows("DETONATE"), "{:?}", ui.texts());
    assert!(ui.texts().iter().any(|t| t.text.starts_with("DECAY        the burst's length")), "{:?}", ui.texts());
}

#[test]
fn dragging_mix_up_wets_the_field_and_the_keyboard_tops_it_out() {
    let mut ui = FaceDriver::of::<Vault>();
    let before = ui.value("mix");
    ui.press("mix").move_to(dy(-40.0));
    assert!(ui.value("mix") > before, "dragging up raises the mix: {before} -> {}", ui.value("mix"));
    ui.release().exit();
    ui.click("mix").key(Key::End);
    assert_eq!(ui.value("mix"), 1.0);
    ui.assert_bracketed();
}

#[test]
fn the_selector_steps_the_machine() {
    let mut ui = FaceDriver::of::<Vault>();
    ui.set("topology", 0.0);
    ui.click("topology").key(Key::End);
    assert_eq!(ui.value("topology"), 4.0, "End picks the last machine");
    assert!(!ui.grabs("dawn").is_empty(), "and VOID's plate follows");
    ui.assert_bracketed();
}
