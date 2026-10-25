use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

// --- batch: the nodes under it use the same model ------------------------------------------

/// Nodes under a batch must use the same model.
#[test]
fn same_model() {
    // `e` uses `smart`, the others `fast`
    assert!(has(here(), "same_model", "batch_models", "graph.nodes[5].model.id"));
}
