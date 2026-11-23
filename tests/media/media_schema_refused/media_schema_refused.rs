use a_graph::run::build;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// An openai node that produces an image and has an output schema is refused when the file is checked, as on gemini:
/// the images api answers no text, so no answer could ever match the schema.
#[test]
fn media_schema_refused() {
    crate::common::run::set_env();
    let yml = crate::common::yml_with(here(), "media_schema_refused", &[("model", 1)]);
    let errs = build(&yml).err().expect("the file was accepted");
    let found: Vec<(&str, String)> = errs.0.iter().map(|e| (e.rule.code(), e.path.to_string())).collect();
    assert!(
        found
            .iter()
            .any(|(rule, path)| *rule == "not_supported" && path.starts_with("graph.nodes[0]")),
        "{found:?}"
    );
}
