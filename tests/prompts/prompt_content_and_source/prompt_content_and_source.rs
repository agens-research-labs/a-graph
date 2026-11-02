use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Prompt with both content and source is refused.
#[test]
fn prompt_content_and_source() {
    assert!(has(
        here(),
        "prompt_content_and_source",
        "exactly_one",
        "resources.prompts[1]"
    ));
}
