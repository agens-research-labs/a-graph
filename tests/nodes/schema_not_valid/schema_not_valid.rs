use crate::common::graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A schema that is a JSON object but not a valid JSON Schema is refused.
#[test]
fn schema_not_valid() {
    let at = (
        "bad_value".to_string(),
        "graph.nodes[0].message.output.schema".to_string(),
    );
    assert!(nodes_of(here(), "not_valid").unwrap_err().contains(&at));
}
