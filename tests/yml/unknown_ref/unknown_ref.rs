use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Citing an undeclared id is an unknown ref.
#[test]
fn unknown_ref() {
    assert!(has(here(), "unknown_ref", "unknown_ref", "graph.nodes[0].model.id"));
}
