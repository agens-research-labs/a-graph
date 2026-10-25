use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The same id twice is a duplicate.
#[test]
fn duplicate_id() {
    assert!(has(here(), "duplicate_id", "duplicate_id", "graph.nodes[2].id"));
}
