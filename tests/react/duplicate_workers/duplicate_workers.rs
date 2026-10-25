use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Two workers with the same name are refused.
#[test]
fn duplicate_workers() {
    assert!(has(here(), "duplicate_workers", "duplicate_id", "graph.edges[1].react.components[1]"));
}
