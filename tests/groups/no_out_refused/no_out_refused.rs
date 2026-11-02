use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A group without out is refused if a node passes its output on.
#[test]
fn no_out_refused() {
    assert!(has(
        here(),
        "message",
        "group_out_required",
        "graph.nodes[4].message.output.node"
    ));
    assert!(has(
        here(),
        "media",
        "group_out_required",
        "graph.nodes[2].media.output[0].node"
    ));
}
