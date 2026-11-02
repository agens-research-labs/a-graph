use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Style conflicts with moderation.
#[test]
fn style_vs_moderation() {
    assert!(has(
        here(),
        "style_vs_moderation",
        "media_provider_conflict",
        "resources.media[1].provider.openai.style"
    ));
}
