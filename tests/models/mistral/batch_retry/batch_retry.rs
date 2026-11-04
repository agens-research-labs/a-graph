use super::*;

/// The mock of the batch API, whose first `status` answers to a poll of the job and first `results` answers to the
/// download of its results are the error `code`; then it answers as `batch_server`. The number of polls is counted.
async fn flaky(status: usize, results: usize, code: u16) -> (String, Arc<Mutex<usize>>) {
    let (handler, _) = batch_server(0, "SUCCESS");
    let (polls, downloads) = (Arc::new(Mutex::new(0usize)), Arc::new(Mutex::new(0usize)));
    let counted = polls.clone();
    let (base, _) = serve(move |req: &Req| {
        if req.method == "GET" {
            let (seen, bad) = if req.path.ends_with("/results") || req.path.ends_with("/content") {
                (&downloads, results)
            } else {
                (&counted, status)
            };
            let mut n = seen.lock().unwrap();
            *n += 1;
            if *n <= bad {
                return Resp::status(code, r#"{"error":{"message":"try later"},"message":"try later"}"#);
            }
        }
        handler(req)
    })
    .await;
    (base, polls)
}

async fn batch(base: &str) -> BatchReturn {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let requests = vec![BatchRequest {
        id: "one".into(),
        invocation: call("one"),
    }];
    client(base).invoke_batch(requests, tx).await
}

/// A poll of the job or a download of its results that fails with an error that may not come again (429, 5xx) is
/// tried again at the next poll, up to 10 failures in a row; any other error ends the batch at once.
#[tokio::test]
async fn batch_retry() {
    // nine failures in a row of the poll, and of the download, do not end the batch
    let (base, polls) = flaky(9, 9, 429).await;
    let results = batch(&base).await.unwrap();
    assert_eq!(results.len(), 1, "{results:?}");
    assert_eq!(*polls.lock().unwrap(), 10);

    // the tenth does, with the error of the provider
    let (base, polls) = flaky(10, 0, 429).await;
    let r = batch(&base).await;
    assert!(
        matches!(&r, Err(ModelError::Provider { status: Some(429), .. })),
        "{r:?}"
    );
    assert_eq!(*polls.lock().unwrap(), 10);

    // an error that comes again is not tried again
    let (base, polls) = flaky(1, 0, 400).await;
    let r = batch(&base).await;
    assert!(
        matches!(&r, Err(ModelError::Provider { status: Some(400), .. })),
        "{r:?}"
    );
    assert_eq!(*polls.lock().unwrap(), 1);
}
