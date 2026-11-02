fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A media with a utility whose source is base64.
#[test]
fn media_path_needed() {
    crate::common::run::set_env();
    let found = crate::common::run::errors(here(), "media_path_needed");
    assert!(
        found
            .iter()
            .any(|(rule, path)| (rule.as_str(), path.as_str())
                == ("media_utility_path", "graph.nodes[0].media.input[0]")),
        "{found:?}"
    );
}
