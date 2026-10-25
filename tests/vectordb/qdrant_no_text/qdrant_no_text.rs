use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A point without the text field is an error, not an empty chunk.
#[tokio::test]
async fn qdrant_no_text() {
    serve_at(19018, |_| Resp::ok(r#"{"embeddings":[[0.5,0.25,1.0]]}"#)).await;
    serve_at(19019, |_| Resp::ok(r#"{"result":{"points":[{"payload":{"other":"x"}}]}}"#)).await;
    let (p, _) = program(here(), "qdrant_no_text", Resp::ok(chat("ok", None))).await;
    let e = p.run_node(entity(&p), user("q")).await.unwrap_err();
    assert!(e.message.contains("payload field `text`"), "{}", e.message);
}
