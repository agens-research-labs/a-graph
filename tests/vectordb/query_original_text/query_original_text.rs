use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// On a node that does not take the message of the user, `message.input.utility` does not change it:
/// the vectordb searches with the message as the user wrote it.
#[tokio::test]
async fn query_original_text() {
    let (embedding_port, emb) = listen(|_| Resp::ok(r#"{"embeddings":[[0.5,0.25,1.0]]}"#)).await;
    let (vectordb_port, _) = listen(|_| Resp::ok(r#"{"result":{"points":[]}}"#)).await;
    let (p, seen) = program_with(
        here(),
        "query_original_text",
        &[("embedding", embedding_port), ("vectordb", vectordb_port)],
        Resp::ok(chat("ok", None)),
    )
    .await;
    let input = NodeInput {
        user_message: Some("payment terms".into()),
        agent_message: Some("the text of the node before".into()),
        ..Default::default()
    };
    p.run_node(entity(&p), input).await.unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&emb.lock().unwrap()[0].body).unwrap()["input"],
        json!(["payment terms"])
    );
    assert_eq!(
        sent(&seen)["messages"][1]["content"],
        "Context:  / From the node before: THE TEXT OF THE NODE BEFORE"
    );
}
