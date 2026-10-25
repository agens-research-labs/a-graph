use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A model without batch is refused for the nodes under a batch.
#[test]
fn model_no_batch() {
    // Ollama has no batch: `check` says so for the nodes of the group
    assert!(has(here(), "model_no_batch", "not_supported", "graph.nodes[4].model"));
}
