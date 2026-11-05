use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A react among the workers of a react, or in a group among them, is refused when the file is
/// checked, before any run.
#[test]
fn nested_react() {
    assert_eq!(
        errors(here(), "nested_react"),
        [
            (
                "react_nested".to_string(),
                "graph.edges[1].react.components[0].react".to_string()
            ),
            (
                "react_nested".to_string(),
                "graph.edges[2].react.components[0].linear[1].fan[0].react".to_string()
            ),
        ]
    );
}
