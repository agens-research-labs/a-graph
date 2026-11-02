use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// An http utility needs `url` and refuses `command` and `args`.
#[test]
fn http_rules() {
    let found = build_errors(&own_yml(here()));
    assert!(
        found.contains(&("missing_field".into(), "utilities[0].url".into())),
        "{found:?}"
    );
    assert!(
        found.contains(&("unexpected_field".into(), "utilities[0].command".into())),
        "{found:?}"
    );
    assert!(
        found.contains(&("unexpected_field".into(), "utilities[0].args".into())),
        "{found:?}"
    );
}
