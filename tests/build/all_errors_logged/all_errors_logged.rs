use crate::common::run::*;
use crate::common::{clean_output};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The errors of every manager are returned and logged.
#[test]
fn all_errors_logged() {
    set_env();
    clean_output("target/test-output/managers_fail.log");
    let e = build(&yml(here(), "all_errors_logged")).err().unwrap();
    let rules = e.rules();
    assert!(rules.contains(&"missing_field") && rules.contains(&"unexpected_field"), "{rules:?}");
    let paths: Vec<String> = e.0.iter().map(|e| e.path.to_string()).collect();
    for want in ["resources.tools[0].command", "external.vectordb[0].collection", "graph.nodes[0].model"] {
        assert!(paths.iter().any(|p| p == want), "{want} not in {paths:?}");
    }
    // the same errors went to the log
    let logged: Vec<String> = log_lines("managers_fail").iter().map(|l| l["path"].as_str().unwrap().to_string()).collect();
    assert_eq!(logged.len(), e.0.len());
}
