//! The face's bindings resolve against the table.

#[test]
fn face_binds_resolve() {
    stave_test::face::binds_resolve::<stave_chime_vst3::Chime>("chime");
}
