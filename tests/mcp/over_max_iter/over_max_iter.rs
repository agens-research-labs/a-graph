use crate::common::run_mcp::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A server asked more than its max iter stops the run.
#[tokio::test]
async fn over_max_iter() {
    let (p, _) = program(here(), "over_max_iter", model(ask("say", json!({ "text": "x" })), true)).await;
    let e = p.run("go").await.unwrap_err();
    assert_eq!(
        (e.rule.code(), e.path.to_string()),
        ("tool_limit", "node(n)".to_string())
    );
    assert!(
        e.message.contains("MCP server `srv`") && e.message.contains("2 times"),
        "{}",
        e.message
    );
}
