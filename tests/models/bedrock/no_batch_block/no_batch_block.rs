use super::*;

/// Without a batch block there is no batch.
#[tokio::test]
async fn no_batch_block() {
    let m = client("http://unused");
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let r = m
        .invoke_batch(
            vec![BatchRequest {
                id: "one".into(),
                invocation: call("q"),
            }],
            tx,
        )
        .await;
    assert!(matches!(r, Err(ModelError::Unsupported { .. })), "{r:?}");
    assert!(
        m.check(&ModelRequirements {
            batch: true,
            ..Default::default()
        })
        .is_err()
    );
    assert!(
        batch_client("http://unused")
            .check(&ModelRequirements {
                batch: true,
                ..Default::default()
            })
            .is_ok()
    );
}
