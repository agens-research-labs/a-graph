use super::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A missing environment variable is an error at build and all are reported.
#[test]
fn env_missing() {
    let e = errors(here(), "env_missing");
    assert_eq!(e.len(), 2, "{e:?}");
    assert!(e.iter().all(|(r, _)| r == "env_missing"));
    assert_eq!(e[0].1, "models[0].api_key");
    assert_eq!(e[1].1, "models[2].api_key");
}
