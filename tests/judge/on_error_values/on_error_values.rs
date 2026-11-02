use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// On error accepts the words stop and node only.
#[test]
fn on_error_values() {
    assert!(has(
        here(),
        "on_error_values",
        "bad_value",
        "graph.nodes[3].validation-node.on_error"
    ));
}
