use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node that declares the same id twice is ambiguous.
#[test]
fn declared_twice() {
    assert!(has(here(), "declared_twice", "duplicate_id", "graph.nodes[0].external"));
}
