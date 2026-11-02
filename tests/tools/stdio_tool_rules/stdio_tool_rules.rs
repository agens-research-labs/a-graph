use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Stdio tool needs a command and refuses a url.
#[test]
fn stdio_tool_rules() {
    assert!(has(
        here(),
        "stdio_tool_rules",
        "missing_field",
        "resources.tools[0].command"
    ));
    assert!(has(
        here(),
        "stdio_tool_rules",
        "unexpected_field",
        "resources.tools[0].url"
    ));
}
