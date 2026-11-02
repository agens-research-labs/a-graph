use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A fixed query is the text that is embedded.
#[tokio::test]
async fn static_query() {
    let (embedding_port, emb) = listen(|_| Resp::ok(r#"{"embeddings":[[0.5,0.25,1.0]]}"#)).await;
    let (vectordb_port, _) = listen(|_| Resp::ok(r#"{"result":{"points":[]}}"#)).await;
    let (p, seen) = program_with(
        here(),
        "static_query",
        &[("embedding", embedding_port), ("vectordb", vectordb_port)],
        Resp::ok(chat("ok", None)),
    )
    .await;
    p.run_node(entity(&p), user("anything")).await.unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&emb.lock().unwrap()[0].body).unwrap()["input"],
        json!(["payment terms"])
    );
    assert_eq!(
        sent(&seen)["messages"][1]["content"],
        "Context:  / Question: anything",
        "no chunk found: the placeholder is empty"
    );
}
