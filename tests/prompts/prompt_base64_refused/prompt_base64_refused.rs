use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Prompt source with base64 is refused.
#[test]
fn prompt_base64_refused() {
    assert!(has(
        here(),
        "prompt_base64_refused",
        "unexpected_field",
        "resources.prompts[1].source.base64"
    ));
}
