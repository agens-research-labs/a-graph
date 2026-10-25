use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The `parameters` of a tool must be a valid JSON Schema.
#[test]
fn tool_params_schema() {
    assert!(has(here(), "tool_params_schema", "bad_value", "resources.tools[0].parameters"));
}
