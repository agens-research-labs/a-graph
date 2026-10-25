use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Unknown provider is a bad value.
#[test]
fn unknown_provider() {
    assert!(has(here(), "unknown_provider", "bad_value", "models[5].provider"));
}
