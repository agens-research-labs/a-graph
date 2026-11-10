use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A branch with a tool in a Bedrock batch gets the error of every provider only: the check of
/// Bedrock no longer refuses the tools a second time.
#[test]
fn bedrock_one_error() {
    let e = errors(here(), "bedrock_one_error");
    assert_eq!(e, [("batch_tools".to_string(), "graph.nodes[2].resources".to_string())]);
}
