fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A `from: node` media that no node before produces.
#[test]
fn media_not_from_node() {
    crate::common::run::set_env();
    let found = crate::common::run::errors(here(), "media_not_from_node");
    assert!(
        found
            .iter()
            .any(|(rule, path)| (rule.as_str(), path.as_str())
                == ("media_not_from_node", "graph.nodes[0].media.input[0]")),
        "{found:?}"
    );
}
