use crate::common::run_mcp::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A server that no node uses is never started.
#[tokio::test]
async fn unused_not_started() {
    let (p, _) = program(here(), "unused_not_started", model(ask("say", json!({ "text": "hi" })), false)).await;
    let out = p.run("go").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(said: hi)");
}
