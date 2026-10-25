use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Media source accepts base64.
#[test]
fn media_base64() {
    assert!(build(here(), "media_base64").is_ok());
}
