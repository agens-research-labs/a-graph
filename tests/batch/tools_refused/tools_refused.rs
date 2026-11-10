use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A branch of a batch with a tool, an MCP server or a skill is refused: the answer of a batch is
/// the last one, its tool calls would never run. The `in` and `out` nodes are not in the batch.
#[test]
fn tools_refused() {
    let mut e = errors(here(), "tools_refused");
    e.sort();
    let want = [
        "graph.nodes[2].resources",
        "graph.nodes[3].resources",
        "graph.nodes[4].resources",
    ]
    .map(|p| ("batch_tools".to_string(), p.to_string()));
    assert_eq!(e, want);
}
