use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The rules reach the groups inside a react.
#[test]
fn rules_in_react() {
    assert!(has(here(), "rules_in_react", "missing_field", "graph.edges[1].react.components[0].linear.in"));
}
