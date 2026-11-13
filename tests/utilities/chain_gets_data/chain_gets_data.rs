use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// What a utility answers reaches the next one as its bytes: nothing is written to disk in between.
#[tokio::test]
async fn chain_gets_data() {
    let (utility, got) = listen(|r| match r.path.as_str() {
        "/first" => Resp::ok(r#"{"mime_type":"image/png","data":"QUJD"}"#),
        _ => Resp::ok(r#"{"text":"converted"}"#),
    })
    .await;
    let (p, seen) = program_with(
        here(),
        "chain_gets_data",
        &[("utility", utility)],
        Resp::ok(chat("ok", None)),
    )
    .await;
    p.run_node(entity(&p), user("what is it")).await.unwrap();
    let got = got.lock().unwrap();
    assert_eq!(got[1].path, "/second");
    let body: Value = serde_json::from_str(&got[1].body).unwrap();
    assert_eq!(body, json!({ "data": "QUJD", "mime_type": "image/png" }));
    let content = sent(&seen)["messages"][1]["content"].as_str().unwrap().to_string();
    assert_eq!(content, "converted\n\nQ: what is it");
}
