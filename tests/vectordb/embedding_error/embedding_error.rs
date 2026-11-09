use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// What the server of the embeddings wrote in its error stays out of the message: it is the detail.
#[tokio::test]
async fn embedding_error() {
    let (embedding_port, _) = listen(|_| Resp::status(500, r#"{"error":"the patient has diabetes"}"#)).await;
    let (vectordb_port, _) = listen(|_| Resp::ok("{}")).await;
    let (p, _) = program_with(
        here(),
        "embedding_error",
        &[("embedding", embedding_port), ("vectordb", vectordb_port)],
        Resp::ok(chat("ok", None)),
    )
    .await;
    let e = p.run_node(entity(&p), user("q")).await.unwrap_err();
    assert_eq!(e.rule.code(), "model_failed");
    assert!(
        e.message.contains("500") && !e.message.contains("diabetes"),
        "{}",
        e.message
    );
    assert_eq!(e.detail.as_deref(), Some("the patient has diabetes"));
}
