use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// An error of the database stops the node and says what the server answered.
#[tokio::test]
async fn database_error() {
    serve_at(19015, |_| Resp::ok(r#"{"embeddings":[[0.5,0.25,1.0]]}"#)).await;
    serve_at(19016, |_| Resp::status(500, "collection broken")).await;
    let (p, _) = program(here(), "database_error", Resp::ok(chat("ok", None))).await;
    let e = p.run_node(entity(&p), user("q")).await.unwrap_err();
    assert_eq!(e.rule.code(), "io");
    assert!(e.message.contains("500") && e.message.contains("collection broken"), "{}", e.message);
}
