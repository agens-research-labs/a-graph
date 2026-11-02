use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Access goes with url only.
#[test]
fn access_needs_url() {
    assert!(has(
        here(),
        "access_needs_url",
        "unexpected_field",
        "resources.prompts[1].source.access"
    ));
}
