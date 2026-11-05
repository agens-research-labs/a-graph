use crate::common::utilities::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A header that names a variable that is not set is an error when the program is built.
#[test]
fn header_env_missing() {
    let missing = manager(&own_yml(here())).missing_env();
    assert_eq!(missing.len(), 1, "{missing:?}");
    assert_eq!(
        (missing[0].rule.code(), missing[0].path.to_string().as_str()),
        ("env_missing", "utilities[0].headers.Authorization")
    );
}
