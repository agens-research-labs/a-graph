use crate::common::run_mcp::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A server of the older revision works too.
#[tokio::test]
async fn older_revision() {
    let (p, _) = program(here(), "older_revision", model(ask("say", json!({ "text": "hi" })), false)).await;
    let out = p.run("go").await.unwrap();
    assert_eq!(items(&out)[0].1, "R(said: hi)");
}
