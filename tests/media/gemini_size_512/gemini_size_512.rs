use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Gemini image size accepts the number 512.
#[test]
fn gemini_size_512() {
    let m = build(here(), "gemini_size_512").unwrap();
    let Origin::Generated { provider: Some(MediaProvider::Gemini(g)), .. } = &m.media.get("cover").unwrap().origin else { panic!() };
    assert_eq!(g.image_size, Some(GeminiImageSize::S512));
}
