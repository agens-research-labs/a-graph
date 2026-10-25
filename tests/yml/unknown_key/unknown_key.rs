use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Unknown key is reported with its path.
#[test]
fn unknown_key() {
    assert!(has(here(), "unknown_key", "unexpected_field", "observability.colour"));
}
