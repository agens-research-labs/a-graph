use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Http tool needs a url and refuses args.
#[test]
fn http_tool_rules() {
    assert!(has(here(), "http_tool_rules", "missing_field", "resources.tools[0].url"));
    assert!(has(here(), "http_tool_rules", "unexpected_field", "resources.tools[0].args"));
}
