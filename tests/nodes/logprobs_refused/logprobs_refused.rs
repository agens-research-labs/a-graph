use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node that asks the logprobs of a model that gives none is refused when the graph is read.
#[test]
fn logprobs_refused() {
    assert_eq!(
        errors(here(), "logprobs_refused"),
        vec![("not_supported".to_string(), "graph.nodes[0].model".to_string())]
    );
}
