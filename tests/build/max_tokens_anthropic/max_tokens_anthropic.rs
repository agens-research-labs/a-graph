use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

// --- the rules of the nodes that need the managers ----------------------------------------

/// Max tokens is mandatory for an anthropic model.
#[test]
fn max_tokens_anthropic() {
    assert!(has(here(), "anthropic", "not_supported", "graph.nodes[0].model"));
    let e = errors(here(), "anthropic");
    assert!(e[0].0 == "not_supported" && e.len() == 1, "one error, not two");
    // the judge too
    assert!(has(here(), "judge", "not_supported", "graph.nodes[3].validation-node.model"));
}
