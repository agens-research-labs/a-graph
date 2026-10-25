use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A fan needs at least one branch.
#[test]
fn fan_not_empty() {
    assert!(has(here(), "fan_not_empty", "bad_value", "graph.edges[2].parallel.fan"));
}
