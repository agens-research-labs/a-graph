use super::*;

/// Vertex has no model in the body and the version there.
#[tokio::test]
async fn vertex_body() {
    let (base, seen) = serve_once().await;
    let m = Anthropic::vertex(&vertex_model("global"), &Path::root()).unwrap().with_messages_url(format!("{base}/rawPredict")).with_token_source(Box::new(StaticToken("goog-token".into())));
    let mut c = call("hi");
    c.system = "be brief".into();
    let r = m.invoke(&c).await.unwrap();
    assert_eq!((r.message.as_deref(), r.info.provider.as_str()), (Some("hello"), "vertex_ai"));

    let req = seen.lock().unwrap()[0].clone();
    assert_eq!(req.path, "/rawPredict");
    assert!(req.headers.contains("authorization: bearer goog-token"), "{}", req.headers);
    // the version is in the body on Vertex AI, not in a header
    assert!(!req.headers.contains("anthropic-version") && !req.headers.contains("x-api-key"), "{}", req.headers);
    let b: Value = serde_json::from_str(&req.body).unwrap();
    assert!(b.get("model").is_none());
    assert_eq!((b["anthropic_version"].as_str(), b["max_tokens"].as_u64(), b["system"].as_str()), (Some("vertex-2023-10-16"), Some(100), Some("be brief")));
}
