use crate::common::run_mcp::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node has its own tools and those of its servers.
#[tokio::test]
async fn own_and_server_tools() {
    let calls = vec![("own", json!({ "city": "Roma" })), ("say", json!({ "text": "x" }))];
    let (p, _) = program(here(), "own_and_server_tools", model(calls, false)).await;
    let out = p.run("go").await.unwrap();
    assert_eq!(items(&out)[0].1, r#"R({"got":{"city":"Roma"}}|said: x)"#);
}
