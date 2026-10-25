use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The question is changed before it is embedded and every chunk before it reaches the prompt.
#[tokio::test]
async fn vectordb_utility() {
    let (_, emb) = serve_at(19926, |_| Resp::ok(r#"{"embeddings":[[0.5,0.25]]}"#)).await;
    serve_at(19925, |_| Resp::ok(r#"{"result":{"points":[{"payload":{"text":"first chunk"}},{"payload":{"text":"second chunk"}}]}}"#)).await;
    let (p, seen) = program(here(), "vectordb_utility", Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("what is it")).await.unwrap();
    assert_eq!(serde_json::from_str::<Value>(&emb.lock().unwrap()[0].body).unwrap()["input"], json!(["[what is it]"]));
    assert_eq!(sent(&seen)["messages"][1]["content"], "C: FIRST CHUNK\n\nSECOND CHUNK / Q: what is it");
}
