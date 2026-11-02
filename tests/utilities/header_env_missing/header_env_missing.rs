use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A header that names a variable that is not set is an error when the program is built.
#[test]
fn header_env_missing() {
    assert!(
        build_errors(&own_yml(here())).contains(&("env_missing".into(), "utilities[0].headers.Authorization".into()))
    );
}
