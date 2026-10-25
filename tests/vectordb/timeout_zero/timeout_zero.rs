use crate::common::external::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// `timeout` of a vectordb is at least 1 second.
#[test]
fn timeout_zero() {
    assert!(has(here(), "timeout_zero", "bad_value", "external.vectordb[0].timeout"));
}
