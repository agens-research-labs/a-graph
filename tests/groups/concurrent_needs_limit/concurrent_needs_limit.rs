use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A concurrent group needs a limit.
#[test]
fn concurrent_needs_limit() {
    assert!(has(here(), "no_limit", "missing_field", "graph.edges[2].concurrent.limit"));
    assert!(loads(here(), "with_limit"));
}
