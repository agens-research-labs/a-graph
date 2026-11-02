use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A judge that passes the problem on needs a node after it.
#[test]
fn judge_needs_next() {
    assert!(has(
        here(),
        "judge_needs_next",
        "validation_no_next",
        "graph.nodes[3].validation-node.on_error"
    ));
}
