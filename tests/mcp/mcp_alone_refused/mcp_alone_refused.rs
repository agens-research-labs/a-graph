use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Mcp servers are refused without calling the model.
#[tokio::test]
async fn mcp_alone_refused() {
    let (p, seen) = program(here(), "mcp_alone_refused", Resp::ok(chat("ok", None))).await;
    let e = p.run_node(entity(&p), user("q")).await.unwrap_err();
    assert_eq!(e.rule.code(), "not_supported");
    assert_eq!(e.path.to_string(), "node(n)");
    assert!(seen.lock().unwrap().is_empty());
}
