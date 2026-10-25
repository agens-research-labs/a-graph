use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Media needs exactly one of source and generate.
#[test]
fn media_source_or_generate() {
    assert!(has(here(), "media_source_or_generate", "exactly_one", "resources.media[0]"));
}
