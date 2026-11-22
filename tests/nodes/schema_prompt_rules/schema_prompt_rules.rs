use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// `schema_prompt` excludes `schema`, carries `max_attempts` (at least 1) and must be a valid JSON Schema.
#[test]
fn schema_prompt_rules() {
    let out = "graph.nodes[0].message.output";
    for (name, rule, key) in [
        ("both", "unexpected_field", "schema_prompt"),
        ("attempts_alone", "unexpected_field", "max_attempts"),
        ("attempts_zero", "bad_value", "max_attempts"),
        ("prompt_invalid", "bad_value", "schema_prompt"),
    ] {
        assert!(
            has(here(), name, rule, &format!("{out}.{key}")),
            "{name}: {:?}",
            errors(here(), name)
        );
    }
    set_env();
    assert!(a_graph::run::build(&yml(here(), "prompt_ok")).is_ok());
}
