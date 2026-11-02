use super::*;

/// Batch is refused until google offers it.
#[tokio::test]
async fn batch_refused() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let r = client("http://127.0.0.1:1")
        .invoke_batch(
            vec![BatchRequest {
                id: "one".into(),
                invocation: call("q"),
            }],
            tx,
        )
        .await;
    assert!(matches!(r, Err(ModelError::Unsupported { .. })), "{r:?}");
}
