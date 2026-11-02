use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

// --- graph.edges: names and links ----------------------------------------------------------

/// A node cited in the edges must exist.
#[test]
fn edge_node_exists() {
    assert!(has(here(), "start_unknown", "unknown_ref", "graph.edges[0].start"));
    assert!(has(
        here(),
        "fan_unknown",
        "unknown_ref",
        "graph.edges[2].parallel[2].fan[0].next"
    ));
}
