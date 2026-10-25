use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The chunks of every vectordb come in the order the node lists them.
#[tokio::test]
async fn two_vectordbs_in_order() {
    serve_at(19008, |_| Resp::ok(r#"{"embeddings":[[0.5,0.25,1.0]]}"#)).await;
    serve_at(19009, |r| match r.method.as_str() {
        "GET" => Resp::ok(r#"{"id":"c-1"}"#),
        _ => Resp::ok(r#"{"documents":[["from chroma"]]}"#),
    }).await;
    serve_at(19010, |_| Resp::ok(r#"{"result":{"points":[{"payload":{"text":"from qdrant"}}]}}"#)).await;
    let (p, seen) = program(here(), "two_vectordbs_in_order", Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("q")).await.unwrap();
    assert_eq!(sent(&seen)["messages"][1]["content"], "Context: from chroma\n\nfrom qdrant / Question: q");
}
