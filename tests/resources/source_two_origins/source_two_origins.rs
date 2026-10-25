use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Source with two origins is refused.
#[test]
fn source_two_origins() {
    assert!(has(here(), "source_two_origins", "exactly_one", "resources.prompts[1].source"));
}
