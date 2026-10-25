use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Word outside the list is a bad value.
#[test]
fn word_not_in_list() {
    assert!(has(here(), "word_not_in_list", "bad_value", "observability.level"));
}
