use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Tool parameters must be an object.
#[test]
fn tool_params_object() {
    assert!(has(here(), "tool_params_object", "bad_value", "resources.tools[0].parameters"));
}
