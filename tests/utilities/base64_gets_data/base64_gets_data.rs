use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A media of the graph in base64 reaches a utility as its bytes, with the type the graph gives, or without one.
#[tokio::test]
async fn base64_gets_data() {
    let (utility, got) = listen(|_| Resp::ok(r#"{"text":"converted"}"#)).await;
    let (p, _) = program_with(
        here(),
        "base64_gets_data",
        &[("utility", utility)],
        Resp::ok(chat("ok", None)),
    )
    .await;
    p.run_node(entity(&p), user("what is it")).await.unwrap();
    let got = got.lock().unwrap();
    let body = |i: usize| serde_json::from_str::<Value>(&got[i].body).unwrap();
    assert_eq!(body(0), json!({ "data": "SGVsbG8=", "mime_type": "application/pdf" }));
    assert_eq!(body(1), json!({ "data": "QUJD" }));
}
