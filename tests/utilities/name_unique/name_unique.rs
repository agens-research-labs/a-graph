use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A name is unique in the whole yml: a utility cannot be called like a model.
#[test]
fn name_unique() {
    assert!(load_errors(&own_yml(here())).iter().any(|(rule, _)| rule == "duplicate_id"));
}
