use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A regular node needs a model.
#[test]
fn node_needs_model() {
    assert!(has(here(), "node_needs_model", "missing_field", "graph.nodes[2].model"));
}
