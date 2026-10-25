use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Placeholders must match what the node declares.
#[test]
fn placeholders_match() {
    assert!(has(here(), "unbound", "placeholder_unbound", "graph.nodes[0].resources.prompt"));
    assert!(has(here(), "unused", "placeholder_unused", "graph.nodes[0].resources.prompt"));
    assert!(has(here(), "malformed", "placeholder_malformed", "graph.nodes[0].resources.prompt"));
    assert!(has(here(), "input_mixed", "placeholder_input_mixed", "graph.nodes[0].resources.prompt"));
}
