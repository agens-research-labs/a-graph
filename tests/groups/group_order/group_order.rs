use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A group needs in and fan each once and in order.
#[test]
fn group_order() {
    assert!(has(here(), "no_in", "missing_field", "graph.edges[2].parallel.in"));
    assert!(has(here(), "wrong_order", "block_shape", "graph.edges[2].parallel"));
    assert!(has(here(), "in_twice", "exactly_one", "graph.edges[2].parallel"));
}
