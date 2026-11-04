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
