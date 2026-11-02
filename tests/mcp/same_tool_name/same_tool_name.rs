use crate::common::run_mcp::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Two tools with the same name in a node are refused before it runs.
#[tokio::test]
async fn same_tool_name() {
    let (p, seen) = program(here(), "same_tool_name", model(city(), false)).await;
    let e = p.run("go").await.unwrap_err();
    assert_eq!(
        (e.rule.code(), e.path.to_string()),
        ("duplicate_id", "node(n)".to_string())
    );
    assert!(e.message.contains("`say`"), "{}", e.message);
    assert!(seen.lock().unwrap().is_empty(), "no node ran");
}
