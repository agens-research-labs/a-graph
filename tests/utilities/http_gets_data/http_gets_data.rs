use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A file on disk reaches a utility on http as its bytes, not as a path it may not be able to read.
#[tokio::test]
async fn http_gets_data() {
    let (utility, got) = listen(|_| Resp::ok(r#"{"text":"converted"}"#)).await;
    let (p, seen) = program_with(
        here(),
        "http_gets_data",
        &[("utility", utility)],
        Resp::ok(chat("ok", None)),
    )
    .await;
    p.run_node(entity(&p), user("what is it")).await.unwrap();
    let body: Value = serde_json::from_str(&got.lock().unwrap()[0].body).unwrap();
    assert_eq!(body, json!({ "data": "UERGLUJZVEVT", "mime_type": "application/pdf" }));
    let content = sent(&seen)["messages"][1]["content"].as_str().unwrap().to_string();
    assert_eq!(content, "converted\n\nQ: what is it");
}
