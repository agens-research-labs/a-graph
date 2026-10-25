
fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A `from: message` media that is not of type `file`.
#[test]
fn from_message_needs_file() {
    crate::common::run::set_env();
    let found = crate::common::run::errors(here(), "from_message_needs_file");
    assert!(found.iter().any(|(rule, path)| (rule.as_str(), path.as_str()) == ("media_not_made", "graph.nodes[0].media.output[0]")), "{found:?}");
}
