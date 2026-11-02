use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Transparency needs png or webp.
#[test]
fn transparency_rules() {
    assert!(has(
        here(),
        "transparency_rules",
        "media_type_unsupported",
        "resources.media[1].generate.background"
    ));
}
