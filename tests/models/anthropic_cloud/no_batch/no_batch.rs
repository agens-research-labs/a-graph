use super::*;

/// Neither platform has a batch.
#[tokio::test]
async fn no_batch() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let v = Anthropic::vertex(&vertex_model("global"), &Path::root()).unwrap();
    let r = v
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
        v.check(&ModelRequirements {
            max_tokens: Some(10),
            batch: true,
            ..Default::default()
        })
        .is_err()
    );
    assert!(
        v.check(&ModelRequirements {
            max_tokens: Some(10),
            ..Default::default()
        })
        .is_ok()
    );

    let a = Anthropic::azure(&azure_model(AzureCredentials::Ambient), &Path::root()).unwrap();
    assert!(
        a.check(&ModelRequirements {
            max_tokens: Some(10),
            batch: true,
            ..Default::default()
        })
        .is_err()
    );
}
