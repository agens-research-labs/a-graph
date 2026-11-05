use crate::common::own_yml;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The destinations are checked before any environment variable is read: a refused host is
/// reported, and the variable of the same model is not even looked at.
#[test]
fn hosts_before_env() {
    let errors = match a_graph::run::build(&own_yml(here())) {
        Ok(_) => panic!("a model whose host is not allowed was accepted"),
        Err(e) => e.0,
    };
    let found: Vec<(&str, String)> = errors.iter().map(|e| (e.rule.code(), e.path.to_string())).collect();
    assert_eq!(found, [("network_not_allowed", "models[0]".to_string())]);
}
