use crate::common::run_mcp::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A server that cannot be reached stops the run before the first node.
#[tokio::test]
async fn cannot_connect() {
    let (p, seen) = program(here(), "cannot_connect", model(city(), false)).await;
    let e = p.run("go").await.unwrap_err();
    assert_eq!((e.rule.code(), e.path.to_string()), ("io", "resources.mcp_servers".to_string()));
    assert!(e.message.contains("MCP server `srv`"), "{}", e.message);
    assert!(seen.lock().unwrap().is_empty());
}
