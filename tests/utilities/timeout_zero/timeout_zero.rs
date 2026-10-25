use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The timeout is at least 1 second.
#[test]
fn timeout_zero() {
    assert!(build_errors(&own_yml(here())).contains(&("bad_value".into(), "utilities[0].timeout".into())));
}
