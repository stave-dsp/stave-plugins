//! The face's bindings resolve against the param table.

#[test]
fn face_binds_resolve() {
    stave_test::face::binds_resolve::<stave_engram_vst3::Engram>("engram");
}
