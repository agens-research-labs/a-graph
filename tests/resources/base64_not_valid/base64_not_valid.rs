use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A `base64` that does not decode is refused when the graph loads.
#[test]
fn base64_not_valid() {
    assert!(has(
        here(),
        "base64_not_valid",
        "bad_value",
        "resources.media[0].source.base64"
    ));
}
