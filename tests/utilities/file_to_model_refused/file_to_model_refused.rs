fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node reads a media of type `file` from the node before, with no utility.
#[test]
fn file_to_model_refused() {
    crate::common::run::set_env();
    let found = crate::common::run::errors(here(), "file_to_model_refused");
    assert!(
        found
            .iter()
            .any(|(rule, path)| (rule.as_str(), path.as_str()) == ("not_supported", "graph.nodes[1].model")),
        "{found:?}"
    );
}
