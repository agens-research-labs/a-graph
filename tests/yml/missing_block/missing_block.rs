use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Missing mandatory block is reported.
#[test]
fn missing_block() {
    assert!(has(here(), "missing_block", "missing_field", "verbose"));
}
