use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Prompt file without content is a bad value.
#[test]
fn prompt_file_no_content() {
    assert!(has(
        here(),
        "prompt_file_no_content",
        "bad_value",
        "resources.prompts[1].source.path"
    ));
}
