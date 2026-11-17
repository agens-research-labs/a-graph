use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Logprobs asked and not given with a text, as by a server that ignores them: the node fails
/// instead of giving an output without them.
#[tokio::test]
async fn logprobs_missing() {
    let (p, _) = program(here(), "logprobs_missing", Resp::ok(chat("ok", None))).await;
    let e = p.run_node(entity(&p), user("q")).await.unwrap_err();
    assert_eq!(
        (e.rule.code(), e.path.to_string()),
        ("model_failed", "node(n)".to_string())
    );
    assert!(e.message.contains("logprobs"), "{}", e.message);
}
