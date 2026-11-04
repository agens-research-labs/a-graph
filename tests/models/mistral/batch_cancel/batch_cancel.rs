use super::*;

/// The cancel of a batch job goes to the path of the Mistral API, and a refusal is an error.
#[tokio::test]
async fn batch_cancel() {
    let (base, seen) = serve(|_| Resp::ok("{}")).await;
    client(&base).cancel_batch("b1").await.unwrap();
    let sent = seen.lock().unwrap()[0].clone();
    assert_eq!(
        (sent.method.as_str(), sent.path.as_str()),
        ("POST", "/v1/batch/jobs/b1/cancel")
    );

    let (base, _) = serve(|_| Resp::status(404, "{}")).await;
    assert!(client(&base).cancel_batch("b1").await.is_err());
}
