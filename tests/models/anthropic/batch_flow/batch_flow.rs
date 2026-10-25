use super::*;

/// A batch is submitted polled and matched by id.
#[tokio::test]
async fn batch_flow() {
    let (handler, polls) = batch_server(2);
    let (base, seen) = serve(handler).await;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let requests = ["one", "two", "three", "four"].map(|id| BatchRequest { id: id.into(), invocation: call(id) }).to_vec();

    let results = client(&base).invoke_batch(requests, tx).await.unwrap();

    // the request: one entry per id, params like a normal call
    let create = seen.lock().unwrap()[0].clone();
    let sent = body(&create);
    assert_eq!(sent["requests"].as_array().unwrap().len(), 4);
    assert_eq!(sent["requests"][0]["custom_id"], "one");
    assert_eq!(sent["requests"][0]["params"]["model"], "claude-haiku-4-5");

    // the status is told at EVERY poll: in_progress, in_progress, ended
    let mut events = Vec::new();
    while let Ok(e) = rx.try_recv() {
        events.push(e);
    }
    let statuses: Vec<_> = events.iter().filter_map(|e| if let BatchEvent::Status { status, .. } = e { Some(status.as_str()) } else { None }).collect();
    assert_eq!(statuses, ["in_progress", "in_progress", "ended"]);
    assert_eq!(*polls.lock().unwrap(), 3);
    assert_eq!(events.first(), Some(&BatchEvent::Submitted { batch_id: "msgbatch_1".into(), requests: 4 }));
    assert_eq!(events.last(), Some(&BatchEvent::Finished { batch_id: "msgbatch_1".into(), outcome: BatchOutcome::Completed }));

    // results are matched by id whatever the order
    let get = |id: &str| results.iter().find(|r| r.id == id).unwrap();
    assert_eq!(get("one").result.as_ref().unwrap().message.as_deref(), Some("first"));
    assert_eq!(get("two").result.as_ref().unwrap().message.as_deref(), Some("second"));
    let three = get("three").result.as_ref().unwrap_err().to_string();
    assert!(three.contains("bad params"), "{three}");
    assert!(matches!(get("four").result, Err(ModelError::Provider { .. })));

    // the results are read from the documented path on our host, not from the URL in the answer
    let paths: Vec<String> = seen.lock().unwrap().iter().map(|r| r.path.clone()).collect();
    assert_eq!(paths.last().unwrap(), "/v1/messages/batches/msgbatch_1/results");
}
