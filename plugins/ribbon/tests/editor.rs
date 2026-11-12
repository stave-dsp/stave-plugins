//! The face's bindings resolve against the parameter table.

#[test]
fn face_binds_resolve() {
    stave_test::face::binds_resolve::<stave_ribbon_vst3::Ribbon>("ribbon");
}
