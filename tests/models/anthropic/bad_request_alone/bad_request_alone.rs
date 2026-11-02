use super::*;

/// A request that cannot be built fails alone.
#[tokio::test]
async fn bad_request_alone() {
    let (handler, _) = batch_server(0);
    let (base, seen) = serve(handler).await;
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut broken = call("broken");
    broken.max_tokens = None;
    let requests = vec![
        BatchRequest {
            id: "one".into(),
            invocation: call("one"),
        },
        BatchRequest {
            id: "bad".into(),
            invocation: broken,
        },
    ];

    let results = client(&base).invoke_batch(requests, tx).await.unwrap();
    assert!(matches!(
        results.iter().find(|r| r.id == "bad").unwrap().result,
        Err(ModelError::Unsupported { .. })
    ));
    assert!(results.iter().find(|r| r.id == "one").unwrap().result.is_ok());
    assert_eq!(body(&seen.lock().unwrap()[0])["requests"].as_array().unwrap().len(), 1);
}
