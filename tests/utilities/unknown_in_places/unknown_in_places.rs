use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// An id that is not in `utilities` is an unknown reference in every place a utility can be written.
#[test]
fn unknown_in_places() {
    let e = a_graph::loader::load(&yml_in(here(), "unknown_in_places")).unwrap_err();
    let at: Vec<String> =
        e.0.iter()
            .filter(|x| x.rule.code() == "unknown_ref")
            .map(|x| x.path.to_string())
            .collect();
    for wanted in [
        "graph.nodes[0].blackboard.write.utility",
        "graph.nodes[0].external.memory[0].read.utility[0]",
        "graph.nodes[0].external.memory[0].write.utility[1]",
        "graph.nodes[0].external.vectordb[0].utility.query",
        "graph.nodes[0].external.vectordb[0].utility.chunks[0]",
    ] {
        assert!(at.iter().any(|p| p == wanted), "no unknown_ref at `{wanted}` in {at:?}");
    }
}
