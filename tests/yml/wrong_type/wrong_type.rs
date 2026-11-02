use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Wrong type is a bad value.
#[test]
fn wrong_type() {
    assert!(has(
        here(),
        "wrong_type",
        "bad_value",
        "graph.nodes[0].model.max_tokens"
    ));
}
