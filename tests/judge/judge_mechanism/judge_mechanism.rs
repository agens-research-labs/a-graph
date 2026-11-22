use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A judge says which mechanism enforces its verdict, `api` or `prompt`; `max_attempts` (at least 1) goes
/// only with `prompt`.
#[test]
fn judge_mechanism() {
    let at = "graph.nodes[1].validation-node";
    for (name, rule, key) in [
        ("missing", "missing_field", "structured_output"),
        ("word", "bad_value", "structured_output"),
        ("attempts_api", "unexpected_field", "max_attempts"),
        ("attempts_zero", "bad_value", "max_attempts"),
    ] {
        assert!(
            has(here(), name, rule, &format!("{at}.{key}")),
            "{name}: {:?}",
            errors(here(), name)
        );
    }
    set_env();
    for name in ["api_ok", "prompt_ok"] {
        if let Err(e) = build(&yml(here(), name)) {
            panic!("{name}: {e}");
        }
    }
}
