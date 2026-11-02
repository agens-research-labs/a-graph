use super::*;

/// A failed batch is a provider error and azure names its deployment.
#[tokio::test]
async fn failed_batch() {
    let (handler, _) = batch_server(0, "failed");
    let (base, _) = serve(handler).await;
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let r = client(&base)
        .invoke_batch(
            vec![BatchRequest {
                id: "one".into(),
                invocation: call("q"),
            }],
            tx,
        )
        .await;
    assert!(
        matches!(&r, Err(ModelError::Provider { message, .. }) if message.contains("failed") && message.contains("the batch was invalid")),
        "{r:?}"
    );

    let (handler, _) = batch_server(0, "completed");
    let (base, seen) = serve(handler).await;
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    // the `batch` block of the yml names the Global-Batch deployment
    let m = OpenAi::azure(&azure_model(azure_key(), Some("my-global-batch")), &Path::root())
        .unwrap()
        .with_base_url(format!("{base}/v1"))
        .with_poll_interval(Duration::from_millis(10));
    m.invoke_batch(
        vec![BatchRequest {
            id: "one".into(),
            invocation: call("q"),
        }],
        tx,
    )
    .await
    .unwrap();
    let upload = seen.lock().unwrap()[0].clone();
    assert!(upload.body.contains("\"model\":\"my-global-batch\""), "{}", upload.body);
}
