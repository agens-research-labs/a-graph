use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The skills of a node have distinct names and leave the tool of the program free.
#[test]
fn distinct_skill_names() {
    assert!(has(
        here(),
        "tool_named_activate",
        "duplicate_id",
        "graph.nodes[0].resources.tools"
    ));
    assert!(has(
        here(),
        "same_name",
        "duplicate_id",
        "graph.nodes[0].resources.skills[1]"
    ));
}
