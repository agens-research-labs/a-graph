use crate::common::clean_output;
use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node that takes only the message of the node before looks for similar exchanges with that message.
#[tokio::test]
async fn similar_by_node_message() {
    clean_output("target/test-memory/similar_by_node_message.json");
    let (embedding_port, emb) = listen(|_| Resp::ok(r#"{"embeddings":[[0.0,1.0]]}"#)).await;
    let (p, _) = program_with(
        here(),
        "similar_by_node_message",
        &[("embedding", embedding_port)],
        Resp::ok(chat("ok", None)),
    )
    .await;
    let input = NodeInput {
        user_message: Some("the text of the user".into()),
        agent_message: Some("the text of the node before".into()),
        ..Default::default()
    };
    p.run_node(entity(&p), input).await.unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&emb.lock().unwrap()[0].body).unwrap()["input"],
        json!(["the text of the node before"])
    );
}
