use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Provider goes with generate.
#[test]
fn provider_needs_generate() {
    assert!(has(
        here(),
        "provider_needs_generate",
        "unexpected_field",
        "resources.media[0].provider"
    ));
}
