use crate::common::own_yml;
use crate::common::run::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A valid yml builds the program.
#[test]
fn valid_builds() {
    set_env();
    let p = build(&own_yml(here())).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!((p.models.len(), p.nodes.len()), (6, 8));
    assert!(p.resources.prompts.get("analyst").is_some() && p.external.vectordb.get("kb").is_some());
    assert!(p.blackboards.ids().any(|i| i == "findings"));
    assert!(p.warnings.is_empty());
}
