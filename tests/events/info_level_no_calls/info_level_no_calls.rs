use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// At `info` the log has the runs, nodes and groups, not the calls to the models.
#[tokio::test]
async fn info_level_no_calls() {
    crate::common::clean_output("target/test-output/info_level_no_calls.log");
    let (p, _) = program(here(), "info_level_no_calls", echo).await;
    p.run("hi").await.unwrap();
    let lines = crate::common::run::log_lines("info_level_no_calls");
    assert!(lines.iter().any(|l| l["code"] == "node.end"));
    assert!(!lines.iter().any(|l| l["code"] == "model.call"));
}
