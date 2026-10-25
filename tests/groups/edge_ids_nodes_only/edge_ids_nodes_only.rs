use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The ids of the edges can be nodes only.
#[test]
fn edge_ids_nodes_only() {
    assert!(has(here(), "edge_ids_nodes_only", "unknown_ref", "graph.edges[3].end"));
}
