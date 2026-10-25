use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The prompt of a judge follows the rules of the placeholders.
#[test]
fn judge_placeholders() {
    assert!(has(here(), "judge_unused", "placeholder_unused", "graph.nodes[3].validation-node.resources.prompt"));
    assert!(has(here(), "judge_unbound", "placeholder_unbound", "graph.nodes[3].validation-node.resources.prompt"));
}
