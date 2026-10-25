use crate::common::loader::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Start is the first entry and end the last.
#[test]
fn start_first_end_last() {
    assert!(has(here(), "end_not_last", "edge_end", "graph.edges[0].end"));
    assert!(has(here(), "start_not_first", "edge_start", "graph.edges[1].start"));
}
