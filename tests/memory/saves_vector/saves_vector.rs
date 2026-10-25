use crate::common::run_node::*;
use crate::common::clean_output;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The vector of the user message is written in the file; the answer has none.
#[tokio::test]
async fn saves_vector() {
    let path = "target/test-memory/saves_vector.json";
    clean_output(path);
    serve_at(19605, |_| Resp::ok(r#"{"embeddings":[[0.5,0.25]]}"#)).await;
    let (p, _) = program(here(), "saves_vector", Resp::ok(chat("the answer", None))).await;
    p.run_node(entity(&p), user("the question")).await.unwrap();
    let records: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(records[0]["chat"][0]["embedding"], json!([0.5, 0.25]));
    assert!(records[0]["chat"][1].get("embedding").is_none());
}
