use super::*;

/// A batch error from the provider is reported.
#[tokio::test]
async fn batch_error() {
    let (base, _) = serve(|_| Resp::status(400, json!({ "error": { "message": "too many requests in batch" } }).to_string())).await;
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let r = client(&base).invoke_batch(vec![BatchRequest { id: "one".into(), invocation: call("one") }], tx).await;
    assert!(matches!(&r, Err(ModelError::Provider { status: Some(400), message, .. }) if message.contains("too many")), "{r:?}");
}
