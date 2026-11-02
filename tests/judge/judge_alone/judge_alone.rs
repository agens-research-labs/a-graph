use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A judge that is the only node of the graph has nothing after it: `on_error: node` has nowhere to go.
#[test]
fn judge_alone() {
    assert!(has(
        here(),
        "judge_alone",
        "validation_no_next",
        "graph.nodes[3].validation-node.on_error"
    ));
}
