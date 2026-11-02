use crate::common::graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A schema written as yml is converted to json.
#[test]
fn schema_yml_to_json() {
    let n = nodes_of(here(), "schema_yml_to_json").unwrap();
    let o = n
        .message()
        .get(n.entity("a").unwrap())
        .unwrap()
        .output
        .as_ref()
        .unwrap();
    assert_eq!(
        o.schema,
        Some(serde_json::json!({ "type": "object", "required": ["x"] }))
    );
}
