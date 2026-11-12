//! The face's bindings resolve against the table.

#[test]
fn face_binds_resolve() {
    stave_test::face::binds_resolve::<stave_smear_vst3::Smear>("smear");
}

/// The face fits a 1000 × 720 pt window.
#[test]
fn face_fits_the_window() {
    use stave_core::Plugin;
    let face = <stave_smear_vst3::Smear as Plugin>::new()
        .editor_face()
        .expect("a face");
    let (w, h) = stave_core::face::Runtime::new(face).size();
    println!("face: {w} x {h}");
    assert!(w <= 1000.0 && h <= 720.0, "face is {w} x {h}");
}
