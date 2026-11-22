use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A react says which mechanism enforces the routing output, `api` or `prompt`; `max_attempts` (at least 1)
/// goes only with `prompt`; the controller has no `schema_prompt` of its own; the model is checked against the
/// routing schema only when its API enforces it.
#[test]
fn react_mechanism() {
    let at = "graph.edges[1].react";
    for (name, rule, path) in [
        ("missing", "missing_field", format!("{at}.structured_output")),
        ("word", "bad_value", format!("{at}.structured_output")),
        ("attempts_api", "unexpected_field", format!("{at}.max_attempts")),
        ("attempts_zero", "bad_value", format!("{at}.max_attempts")),
        (
            "controller_prompt",
            "unexpected_field",
            "graph.nodes[1].message.output.schema_prompt".to_string(),
        ),
        ("media_api", "not_supported", "graph.nodes[1].model".to_string()),
    ] {
        assert!(has(here(), name, rule, &path), "{name}: {:?}", errors(here(), name));
    }
    set_env();
    for name in ["api_ok", "prompt_ok", "media_prompt"] {
        if let Err(e) = build(&yml(here(), name)) {
            panic!("{name}: {e}");
        }
    }
}
