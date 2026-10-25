use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A schema gives json.
#[tokio::test]
async fn schema_gives_json() {
    let (p, seen) = program(here(), "schema_gives_json", Resp::ok(chat("{\"a\": 1}", None))).await;
    let out = p.run_node(entity(&p), user("q")).await.unwrap();
    assert_eq!(sent(&seen)["format"], json!({ "type": "object" }));
    assert_eq!(out.assistant.message, Some(Content::Json(json!({ "a": 1 }))));
}
