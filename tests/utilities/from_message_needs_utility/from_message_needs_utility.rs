fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A `from: message` media with no utility.
#[test]
fn from_message_needs_utility() {
    crate::common::run::set_env();
    let found = crate::common::run::errors(here(), "from_message_needs_utility");
    assert!(
        found
            .iter()
            .any(|(rule, path)| (rule.as_str(), path.as_str())
                == ("media_needs_utility", "graph.nodes[0].media.output[0]")),
        "{found:?}"
    );
}
