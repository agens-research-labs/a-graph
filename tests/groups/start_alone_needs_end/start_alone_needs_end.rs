
fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Only a graph of one node may leave out the `end`.
#[test]
fn start_alone_needs_end() {
    let found = crate::common::run::errors(here(), "start_alone_needs_end");
    assert!(found.contains(&("edge_end".to_string(), "graph.edges".to_string())), "{found:?}");
}
