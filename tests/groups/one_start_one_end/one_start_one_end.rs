use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

// --- graph.edges: the rules ----------------------------------------------------------------

/// Exactly one start and one end.
#[test]
fn one_start_one_end() {
    assert!(has(here(), "two_ends", "edge_end", "graph.edges[4].end"));
    assert!(has(here(), "no_start", "edge_start", "graph.edges"));
    assert!(has(here(), "no_end", "edge_end", "graph.edges"));
}
