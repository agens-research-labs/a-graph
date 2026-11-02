use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A missing environment variable is reported.
#[test]
fn env_missing() {
    assert_eq!(
        errors(here(), "env_missing")
            .iter()
            .map(|(r, _)| r.as_str())
            .collect::<Vec<_>>(),
        ["env_missing"]
    );
}
