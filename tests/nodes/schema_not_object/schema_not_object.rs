use crate::common::graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A schema that is not json or not an object is refused.
#[test]
fn schema_not_object() {
    let at = (
        "bad_value".to_string(),
        "graph.nodes[0].message.output.schema".to_string(),
    );
    assert!(nodes_of(here(), "not_json").unwrap_err().contains(&at));
    assert!(nodes_of(here(), "not_object").unwrap_err().contains(&at));
}
