use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Misspelled key is unknown and the real one is missing.
#[test]
fn misspelled_key() {
    assert!(has(here(), "misspelled_key", "unexpected_field", "models[5].hots"));
    assert!(has(here(), "misspelled_key", "missing_field", "models[5].host"));
}
