use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The `parameters` of a tool must be convertible to JSON: a key that is not a string is an error.
#[test]
fn tool_params_not_json() {
    assert!(has(
        here(),
        "tool_params_not_json",
        "bad_value",
        "resources.tools[0].parameters"
    ));
}
