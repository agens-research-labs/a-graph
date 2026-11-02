use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node cites its tools by id and the cap belongs to the tool.
#[test]
fn node_tools_by_id() {
    assert!(has(
        here(),
        "tool_no_max_iter",
        "missing_field",
        "resources.tools[0].max_iter"
    ));
    assert!(has(
        here(),
        "tool_no_timeout",
        "missing_field",
        "resources.tools[0].timeout"
    ));
    assert!(has(
        here(),
        "mcp_no_max_iter",
        "missing_field",
        "resources.mcp_servers[0].max_iter"
    ));
    assert!(has(
        here(),
        "mcp_no_timeout",
        "missing_field",
        "resources.mcp_servers[0].timeout"
    ));
    assert!(has(
        here(),
        "mcp_unknown",
        "unknown_ref",
        "graph.nodes[0].resources.mcp_servers[0]"
    ));
    assert!(has(
        here(),
        "unknown",
        "unknown_ref",
        "graph.nodes[0].resources.tools[0]"
    ));
    assert!(
        errors(here(), "old_form")
            .iter()
            .any(|(_, p)| p == "graph.nodes[0].resources.tools[0]")
    );
}
