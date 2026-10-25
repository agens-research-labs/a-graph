use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A stdio utility needs `command` and refuses `url`.
#[test]
fn stdio_rules() {
    let found = build_errors(&own_yml(here()));
    assert!(found.contains(&("missing_field".into(), "utilities[0].command".into())), "{found:?}");
    assert!(found.contains(&("unexpected_field".into(), "utilities[0].url".into())), "{found:?}");
}
