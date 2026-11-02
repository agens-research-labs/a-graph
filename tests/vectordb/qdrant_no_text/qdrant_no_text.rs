use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A point without the text field is an error, not an empty chunk.
#[tokio::test]
async fn qdrant_no_text() {
    let (embedding_port, _) = listen(|_| Resp::ok(r#"{"embeddings":[[0.5,0.25,1.0]]}"#)).await;
    let (vectordb_port, _) = listen(|_| Resp::ok(r#"{"result":{"points":[{"payload":{"other":"x"}}]}}"#)).await;
    let (p, _) = program_with(
        here(),
        "qdrant_no_text",
        &[("embedding", embedding_port), ("vectordb", vectordb_port)],
        Resp::ok(chat("ok", None)),
    )
    .await;
    let e = p.run_node(entity(&p), user("q")).await.unwrap_err();
    assert!(e.message.contains("payload field `text`"), "{}", e.message);
}
