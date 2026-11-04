use super::*;

/// The cancel of a batch job goes to the path of the Anthropic API, and a refusal is an error.
#[tokio::test]
async fn batch_cancel() {
    let (base, seen) = serve(|_| Resp::ok("{}")).await;
    client(&base).cancel_batch("b1").await.unwrap();
    let sent = seen.lock().unwrap()[0].clone();
    assert_eq!(
        (sent.method.as_str(), sent.path.as_str()),
        ("POST", "/v1/messages/batches/b1/cancel")
    );

    let (base, _) = serve(|_| Resp::status(404, "{}")).await;
    assert!(client(&base).cancel_batch("b1").await.is_err());
}

/// A mock that refuses every cancel (409) and answers `state` to a read of the job at `/v1/messages/batches/b1`.
async fn refusing(state: &'static str) -> String {
    let (base, _) = serve(move |req: &Req| match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/v1/messages/batches/b1") => Resp::ok(json!({ "processing_status": state }).to_string()),
        _ => Resp::status(
            409,
            r#"{"error":{"message":"cannot cancel"},"message":"cannot cancel"}"#,
        ),
    })
    .await;
    base
}

/// A refused cancel is no failure when the job has already ended or is already being cancelled: nothing is left
/// running. When the job still runs, the refusal stands.
#[tokio::test]
async fn batch_cancel_refused() {
    for state in ["ended", "canceling"] {
        let base = refusing(state).await;
        let r = client(&base).cancel_batch("b1").await;
        assert!(r.is_ok(), "{state}: {r:?}");
    }
    let base = refusing("in_progress").await;
    assert!(client(&base).cancel_batch("b1").await.is_err());
}
