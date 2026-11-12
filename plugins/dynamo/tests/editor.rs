//! The face's bindings resolve against the table.

#[test]
fn face_binds_resolve() {
    stave_test::face::binds_resolve::<stave_dynamo_vst3::Dynamo>("dynamo");
}
