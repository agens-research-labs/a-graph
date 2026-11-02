use crate::common::resources::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Provider with two families is refused.
#[test]
fn two_families() {
    assert!(has(
        here(),
        "two_families",
        "exactly_one",
        "resources.media[1].provider"
    ));
}
