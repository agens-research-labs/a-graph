use super::*;

/// A batch is uploaded created polled and matched by id.
#[tokio::test]
async fn batch_flow() {
    let (handler, polls) = batch_server(2, "completed");
    let (base, seen) = serve(handler).await;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let requests = ["one", "two", "three", "four"]
        .map(|id| BatchRequest {
            id: id.into(),
            invocation: call(id),
        })
        .to_vec();

    let results = client(&base).invoke_batch(requests, tx).await.unwrap();

    let requests = seen.lock().unwrap().clone();
    let upload = &requests[0];
    assert_eq!((upload.method.as_str(), upload.path.as_str()), ("POST", "/v1/files"));
    assert!(upload.headers.contains("multipart/form-data"));
    assert!(upload.body.contains("purpose") && upload.body.contains("batch"));
    // one JSON line per request, the way the Batch API wants them
    let lines: Vec<Value> = upload
        .body
        .lines()
        .filter(|l| l.starts_with('{'))
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines.len(), 4);
    assert_eq!(
        (
            lines[0]["custom_id"].as_str(),
            lines[0]["method"].as_str(),
            lines[0]["url"].as_str()
        ),
        (Some("one"), Some("POST"), Some("/v1/responses"))
    );
    assert_eq!(lines[0]["body"]["model"], "gpt-5");
    assert_eq!(
        body(&requests[1]),
        json!({ "input_file_id": "file-in", "endpoint": "/v1/responses", "completion_window": "24h" })
    );

    let mut events = Vec::new();
    while let Ok(e) = rx.try_recv() {
        events.push(e);
    }
    let statuses: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let BatchEvent::Status { status, .. } = e {
                Some(status.as_str())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(statuses, ["in_progress", "in_progress", "completed"]);
    assert_eq!(*polls.lock().unwrap(), 3);
    assert_eq!(
        events.first(),
        Some(&BatchEvent::Submitted {
            batch_id: "batch_1".into(),
            requests: 4
        })
    );
    assert_eq!(
        events.last(),
        Some(&BatchEvent::Finished {
            batch_id: "batch_1".into(),
            outcome: BatchOutcome::Completed
        })
    );

    let get = |id: &str| results.iter().find(|r| r.id == id).unwrap();
    assert_eq!(get("one").result.as_ref().unwrap().message.as_deref(), Some("first"));
    assert_eq!(get("two").result.as_ref().unwrap().message.as_deref(), Some("second"));
    assert!(
        get("three")
            .result
            .as_ref()
            .unwrap_err()
            .to_string()
            .contains("bad params")
    );
    assert!(get("four").result.as_ref().unwrap_err().to_string().contains("expired"));
}
