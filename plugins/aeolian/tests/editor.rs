//! The face's bindings resolve against the param table.

#[test]
fn face_binds_resolve() {
    stave_test::face::binds_resolve::<stave_aeolian_vst3::Aeolian>("aeolian");
}
