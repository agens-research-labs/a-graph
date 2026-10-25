use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The same name in two different kinds is a duplicate: a name is unique in the whole yml.
#[test]
fn id_unique_across_kinds() {
    let found = errors(here(), "id_unique_across_kinds");
    assert!(found.iter().any(|(rule, _)| rule == "duplicate_id"), "{found:?}");
}
