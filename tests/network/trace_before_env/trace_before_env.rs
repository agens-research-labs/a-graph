use crate::common::own_yml;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The trace endpoint is checked before the variables of its headers are read: a refused host is
/// reported, and whether the variable is set is never told.
#[test]
fn trace_before_env() {
    let errors = match a_graph::run::build(&own_yml(here())) {
        Ok(_) => panic!("a trace endpoint whose host is not allowed was accepted"),
        Err(e) => e.0,
    };
    let found: Vec<(&str, String)> = errors.iter().map(|e| (e.rule.code(), e.path.to_string())).collect();
    assert_eq!(found, [("network_not_allowed", "trace.endpoint".to_string())]);
}
