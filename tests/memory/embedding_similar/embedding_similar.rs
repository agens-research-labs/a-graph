use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// With an embedding the exchange nearest to the new message is the one that comes back.
#[tokio::test]
async fn embedding_similar() {
    // the new message looks like the taxes: its vector is [0, 1]
    let (embedding_port, emb) = listen(|_| Resp::ok(r#"{"embeddings":[[0.0,1.0]]}"#)).await;
    let (p, seen) = program_with(here(), "embedding_similar", &[("embedding", embedding_port)], Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("when do I pay?")).await.unwrap();
    let contents: Vec<String> = sent(&seen)["messages"].as_array().unwrap().iter().skip(1).map(|m| m["content"].as_str().unwrap().into()).collect();
    assert_eq!(contents, ["about taxes", "taxes are due", "Question: when do I pay?"]);
    assert_eq!(serde_json::from_str::<Value>(&emb.lock().unwrap()[0].body).unwrap()["input"], json!(["when do I pay?"]));
}
