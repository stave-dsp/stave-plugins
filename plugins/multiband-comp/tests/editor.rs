//! The face, as a person uses it (the generic face laws run in
//! `checks.rs`).

use stave_multiband_comp_vst3::MultibandComp;
use stave_test::face::{dy, FaceDriver};

/// The band page set's key (the face's `BANDS_KEY`).
const BANDS: u32 = 1;

/// The face fits a 1000 × 720 pt window.
#[test]
fn face_fits_the_window() {
    let ui = FaceDriver::of::<MultibandComp>();
    let (w, h) = ui.size();
    assert!(w <= 1000.0 && h <= 720.0, "face is {w} x {h}");
}

/// The LOW / MID / HIGH tabs flip the band page: one band's controls at
/// a time, the rest of the face fixed.
#[test]
fn the_band_tabs_show_one_band_at_a_time() {
    let mut ui = FaceDriver::of::<MultibandComp>();
    assert_eq!(ui.page(BANDS), 0);
    assert!(!ui.grabs("lo threshold").is_empty(), "LOW shows first");
    assert!(ui.grabs("mid threshold").is_empty());
    let tab = ui
        .texts()
        .into_iter()
        .find(|t| t.text == "MID" && !t.live)
        .expect("a MID tab");
    let b = tab.bounds;
    ui.click((b.x + b.w / 2.0, b.y + b.h / 2.0));
    assert_eq!(ui.page(BANDS), 1);
    assert!(ui.grabs("lo threshold").is_empty());
    assert!(!ui.grabs("mid threshold").is_empty(), "MID shows its compressor");
    assert!(!ui.grabs("low/mid").is_empty(), "the crossovers stay");
}

/// Dragging a band's threshold down lowers it, bracketed as one gesture.
#[test]
fn dragging_the_threshold_moves_it() {
    let mut ui = FaceDriver::of::<MultibandComp>();
    let before = ui.value("lo threshold");
    ui.press("lo threshold").move_to(dy(40.0)).release();
    let after = ui.value("lo threshold");
    assert!(after < before, "threshold {before} -> {after}");
    ui.assert_bracketed();
}
