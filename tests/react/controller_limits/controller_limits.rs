use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

// --- react ---------------------------------------------------------------------------------

/// A react controller has no tools blackboard or output schema.
#[test]
fn controller_limits() {
    assert!(has(here(), "with_tools", "unexpected_field", "graph.nodes[1].resources.tools"));
    assert!(has(here(), "with_blackboard", "unexpected_field", "graph.nodes[1].blackboard"));
    assert!(has(here(), "with_schema", "unexpected_field", "graph.nodes[1].message.output.schema"));
}
