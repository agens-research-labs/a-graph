use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A group with out may pass the output on.
#[test]
fn group_with_out() {
    assert!(loads(here(), "group_with_out"));
}
