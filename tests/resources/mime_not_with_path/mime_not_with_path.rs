use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// `mime_type` goes with `url` or `base64` only: a file has the type of its extension.
#[test]
fn mime_not_with_path() {
    assert!(has(
        here(),
        "mime_not_with_path",
        "unexpected_field",
        "resources.media[0].source.mime_type"
    ));
}
