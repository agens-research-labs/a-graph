
fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A media of type `file` that has a source.
#[test]
fn file_has_no_source() {
    crate::common::run::set_env();
    let found = crate::common::run::errors(here(), "file_has_no_source");
    assert!(found.iter().any(|(rule, path)| (rule.as_str(), path.as_str()) == ("exactly_one", "resources.media[0]")), "{found:?}");
}
