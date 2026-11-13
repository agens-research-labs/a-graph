use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// `mime_type` goes with `base64` only.
#[test]
fn mime_needs_base64() {
    assert!(has(
        here(),
        "mime_needs_base64",
        "unexpected_field",
        "resources.media[0].source.mime_type"
    ));
}
