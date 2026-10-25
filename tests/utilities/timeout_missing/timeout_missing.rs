use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The timeout is mandatory, like every cap.
#[test]
fn timeout_missing() {
    assert!(load_errors(&own_yml(here())).contains(&("missing_field".into(), "utilities[0].timeout".into())));
}
