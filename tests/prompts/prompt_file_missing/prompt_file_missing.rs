use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Prompt file that does not exist is an io error.
#[test]
fn prompt_file_missing() {
    assert!(has(
        here(),
        "prompt_file_missing",
        "io",
        "resources.prompts[1].source.path"
    ));
}
