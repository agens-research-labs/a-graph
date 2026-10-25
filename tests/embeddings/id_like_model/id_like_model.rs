use crate::common::embeddings::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// An embedding with the id of a model is a duplicate: a name is unique in the whole yml.
#[test]
fn id_like_model() {
    let found = load_errors(&own_yml(here()));
    assert!(found.iter().any(|(rule, _)| rule == "duplicate_id"), "{found:?}");
}
