use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// An error of the database stops the node with the status; what the server answered is the detail.
#[tokio::test]
async fn database_error() {
    let (embedding_port, _) = listen(|_| Resp::ok(r#"{"embeddings":[[0.5,0.25,1.0]]}"#)).await;
    let (vectordb_port, _) = listen(|_| Resp::status(500, "collection broken")).await;
    let (p, _) = program_with(
        here(),
        "database_error",
        &[("embedding", embedding_port), ("vectordb", vectordb_port)],
        Resp::ok(chat("ok", None)),
    )
    .await;
    let e = p.run_node(entity(&p), user("q")).await.unwrap_err();
    assert_eq!(e.rule.code(), "io");
    assert!(
        e.message.contains("500") && !e.message.contains("collection broken"),
        "{}",
        e.message
    );
    assert_eq!(e.detail.as_deref(), Some("collection broken"));
}
