use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// What the model cannot do is found by check before any call.
#[test]
fn check_before_call() {
    // Ollama has no media output
    assert!(has(
        here(),
        "check_before_call",
        "not_supported",
        "graph.nodes[2].model"
    ));
}
