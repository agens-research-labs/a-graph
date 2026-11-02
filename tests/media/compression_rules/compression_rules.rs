use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Compression needs jpeg or webp and a value up to 100.
#[test]
fn compression_rules() {
    assert!(has(
        here(),
        "compression_rules",
        "bad_value",
        "resources.media[1].provider.openai.output_compression"
    ));
    assert!(has(
        here(),
        "compression_rules",
        "media_type_unsupported",
        "resources.media[1].provider.openai.output_compression"
    ));
}
