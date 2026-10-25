use super::*;

/// A batch goes inline is polled and matched by id.
#[tokio::test]
async fn batch_inline() {
    let (handler, polls) = batch_server(2, "SUCCESS");
    let (base, seen) = serve(handler).await;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let requests = ["one", "two", "three", "four"].map(|id| BatchRequest { id: id.into(), invocation: call(id) }).to_vec();
    let results = client(&base).invoke_batch(requests, tx).await.unwrap();

    let create = body(&seen.lock().unwrap()[0]);
    assert_eq!((create["endpoint"].as_str(), create["model"].as_str()), (Some("/v1/chat/completions"), Some("mistral-small-latest")));
    assert_eq!(create["requests"].as_array().unwrap().len(), 4);
    assert_eq!(create["requests"][0]["custom_id"], "one");
    assert_eq!(create["requests"][0]["body"]["messages"][0]["role"], "user");

    let mut events = Vec::new();
    while let Ok(e) = rx.try_recv() {
        events.push(e);
    }
    let statuses: Vec<_> = events.iter().filter_map(|e| if let BatchEvent::Status { status, .. } = e { Some(status.as_str()) } else { None }).collect();
    assert_eq!(statuses, ["RUNNING", "RUNNING", "SUCCESS"]);
    assert_eq!(*polls.lock().unwrap(), 3);
    assert_eq!(events.first(), Some(&BatchEvent::Submitted { batch_id: "job_1".into(), requests: 4 }));
    assert_eq!(events.last(), Some(&BatchEvent::Finished { batch_id: "job_1".into(), outcome: BatchOutcome::Completed }));

    let get = |id: &str| results.iter().find(|r| r.id == id).unwrap();
    assert_eq!(get("one").result.as_ref().unwrap().message.as_deref(), Some("first"));
    assert_eq!(get("two").result.as_ref().unwrap().message.as_deref(), Some("second"));
    assert!(get("three").result.as_ref().unwrap_err().to_string().contains("bad params"));
    assert!(get("four").result.as_ref().unwrap_err().to_string().contains("timed out"));
}
