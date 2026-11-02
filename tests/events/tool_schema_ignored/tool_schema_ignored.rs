use crate::common::run_mcp::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A tool of an MCP server whose schema is not valid is not checked: the run warns once and goes on.
#[tokio::test]
async fn tool_schema_ignored() {
    crate::common::clean_output("target/test-output/tool_schema_ignored.log");
    let (p, _) = program(
        here(),
        "tool_schema_ignored",
        model(vec![("say", json!({ "text": "x" }))], false),
    )
    .await;
    let out = p.run("go").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(said: x)");
    let lines = crate::common::run::log_lines("tool_schema_ignored");
    let warns: Vec<_> = lines.iter().filter(|l| l["code"] == "tool_schema_ignored").collect();
    assert_eq!(warns.len(), 1, "{warns:?}");
    assert_eq!(warns[0]["level"], "warn");
    assert!(warns[0]["message"].as_str().unwrap().contains("`odd`"), "{}", warns[0]);
}
