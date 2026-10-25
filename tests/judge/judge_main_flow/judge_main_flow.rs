use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A judge stands in the main flow never in a group or a react.
#[test]
fn judge_main_flow() {
    let at = "graph.nodes[3].validation-node";
    for name in ["as_group_in", "in_fan", "as_group_out", "in_nested_group", "in_react", "as_react_controller"] {
        assert!(has(here(), name, "validation_in_group", at), "{name}");
    }
    assert!(loads(here(), "in_main_flow"));
}
