use crate::common::graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A validation node has only id and validation node.
#[test]
fn judge_only_id() {
    for block in ["model", "message", "resources", "blackboard", "media", "external"] {
        let e = nodes_of(here(), block).unwrap_err();
        assert!(
            e.contains(&("unexpected_field".into(), format!("graph.nodes[3].{block}"))),
            "{block}: {e:?}"
        );
    }
}
