use super::*;

/// The cancel of a batch job goes to the path of the Bedrock API, and a refusal is an error.
#[tokio::test]
async fn batch_cancel() {
    let (base, seen) = serve(|_| Resp::ok("{}")).await;
    // the API takes the id of the job without the rest of the ARN
    client(&base).cancel_batch("job42").await.unwrap();
    let sent = seen.lock().unwrap()[0].clone();
    assert_eq!(
        (sent.method.as_str(), sent.path.as_str()),
        ("POST", "/model-invocation-job/job42/stop")
    );

    let (base, _) = serve(|_| Resp::status(404, "{}")).await;
    assert!(client(&base).cancel_batch("job42").await.is_err());
}

/// A mock that refuses every cancel (409) and answers `state` to a read of the job at `/model-invocation-job/job42`.
async fn refusing(state: &'static str) -> String {
    let (base, _) = serve(move |req: &Req| match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/model-invocation-job/job42") => Resp::ok(job(state)),
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
    for state in [
        "Completed",
        "PartiallyCompleted",
        "Failed",
        "Stopped",
        "Expired",
        "Stopping",
    ] {
        let base = refusing(state).await;
        let r = client(&base).cancel_batch("job42").await;
        assert!(r.is_ok(), "{state}: {r:?}");
    }
    let base = refusing("InProgress").await;
    assert!(client(&base).cancel_batch("job42").await.is_err());
}
