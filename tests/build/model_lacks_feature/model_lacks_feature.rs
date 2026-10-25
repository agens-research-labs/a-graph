use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A model that lacks what the node asks is refused.
#[test]
fn model_lacks_feature() {
    // Anthropic produces text only
    assert!(has(here(), "model_lacks_feature", "not_supported", "graph.nodes[1].model"));
}
