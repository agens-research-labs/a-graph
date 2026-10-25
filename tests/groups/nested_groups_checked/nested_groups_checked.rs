use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Groups are nestable and the nested ones are checked too.
#[test]
fn nested_groups_checked() {
    assert!(has(here(), "nested_groups_checked", "unknown_ref", "graph.edges[2].parallel[2].fan[1].linear[0].in"));
}
