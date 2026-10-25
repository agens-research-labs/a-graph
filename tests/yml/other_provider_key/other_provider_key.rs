use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A key of another provider is not allowed.
#[test]
fn other_provider_key() {
    assert!(has(here(), "other_provider_key", "unexpected_field", "models[5].api_key"));
}
