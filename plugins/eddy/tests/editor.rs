//! The face, as a person uses it: the tabs choose what runs and the
//! plate under them follows (the generic face laws run in `checks.rs`).

use stave_eddy_vst3::Eddy;
use stave_test::face::{dy, FaceDriver};

/// Flip the pager to page `page` (0 MOTION, 1 ALGO, 2 FIELD).
fn page(ui: &mut FaceDriver, page: usize) {
    let at = ui.grabs("pages")[page].area.centre();
    ui.click(at);
}

/// Click tab `step` of the tab row bound to `param`.
fn tab(ui: &mut FaceDriver, param: &str, step: usize) {
    let at = ui.grabs(param)[step].area.centre();
    ui.click(at);
}

#[test]
fn clicking_an_algorithm_tab_runs_it_and_shows_its_knobs() {
    let mut ui = FaceDriver::of::<Eddy>();
    page(&mut ui, 1);
    assert_eq!(ui.value("algo"), 0.0);
    assert!(!ui.grabs("heat").is_empty(), "CIRCUIT's knobs show under its tab");
    assert!(ui.grabs("shift").is_empty(), "SHIFT's knobs are not on the face yet");

    tab(&mut ui, "algo", 2);
    assert_eq!(ui.value("algo"), 2.0, "the SHIFT tab selects the SHIFT algorithm");
    for knob in ["shift", "grain", "splice"] {
        assert!(!ui.grabs(knob).is_empty(), "{knob} shows under the SHIFT tab");
    }
    for knob in ["heat", "sag", "inertia", "orbit"] {
        assert!(ui.grabs(knob).is_empty(), "{knob} belongs to another tab");
    }

    tab(&mut ui, "algo", 3);
    assert_eq!(ui.value("algo"), 3.0);
    assert!(!ui.grabs("orbit").is_empty() && ui.grabs("shift").is_empty());
    ui.assert_bracketed();
}

#[test]
fn clicking_a_field_tab_picks_the_voice_and_its_knobs() {
    let mut ui = FaceDriver::of::<Eddy>();
    page(&mut ui, 2);
    tab(&mut ui, "diffusion", 1);
    assert_eq!(ui.value("diffusion"), 1.0, "the PRISM tab selects PRISM");
    assert!(!ui.grabs("bend").is_empty() && !ui.grabs("smear").is_empty());
    assert!(ui.grabs("density").is_empty(), "PLATE's knobs left with its tab");
    assert!(!ui.grabs("diffuse").is_empty(), "the couplings stay under every voice");
    ui.assert_bracketed();
}

#[test]
fn the_time_knob_drags_the_delay() {
    let mut ui = FaceDriver::of::<Eddy>();
    let before = ui.value("time");
    ui.press("time").move_to(dy(-40.0));
    assert!(ui.value("time") > before, "dragging up lengthens the delay");
    ui.release().exit();
    ui.assert_bracketed();
}
