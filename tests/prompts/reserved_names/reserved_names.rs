use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The names of the message placeholders cannot be the id of a blackboard the node reads.
#[test]
fn reserved_names() {
    assert!(has(here(), "reserved_names", "duplicate_id", "graph.nodes[0].external"));
}
