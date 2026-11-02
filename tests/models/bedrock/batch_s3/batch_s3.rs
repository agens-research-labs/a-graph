use super::*;

/// A batch goes to s3 runs as a job and is matched by id.
#[tokio::test]
async fn batch_s3() {
    let (handler, polls) = batch_server(2, "Completed", true);
    let (base, seen) = serve(handler).await;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut with_media = call("three");
    with_media.max_tokens = Some(50);
    with_media.system = "be brief".into();
    let requests = vec![
        BatchRequest {
            id: "one".into(),
            invocation: call("one"),
        },
        BatchRequest {
            id: "two".into(),
            invocation: call("two"),
        },
        BatchRequest {
            id: "three".into(),
            invocation: with_media,
        },
    ];
    let results = batch_client(&base).invoke_batch(requests, tx).await.unwrap();

    let requests = seen.lock().unwrap().clone();
    // the input file: one Converse body per record
    let upload = requests.iter().find(|r| r.method == "PUT").unwrap();
    assert!(
        upload.path.starts_with("/my-in/jobs/a-graph-") && upload.path.contains(".jsonl"),
        "{}",
        upload.path
    );
    let lines: Vec<Value> = upload.body.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0]["recordId"], "REC00000000");
    assert_eq!(
        lines[0]["modelInput"]["messages"],
        json!([{ "role": "user", "content": [{ "text": "one" }] }])
    );
    assert_eq!(lines[2]["modelInput"]["system"], json!([{ "text": "be brief" }]));
    assert_eq!(lines[2]["modelInput"]["inferenceConfig"], json!({ "maxTokens": 50 }));

    // the job: the model, the role, the Converse invocation type, the input file and the output folder
    let create = requests.iter().find(|r| r.method == "POST").unwrap();
    let c = body(create);
    assert_eq!(c["modelId"], "anthropic.claude-sonnet-4-5");
    assert_eq!(c["roleArn"], "arn:aws:iam::123456789012:role/batch");
    assert_eq!(c["modelInvocationType"], "Converse");
    assert_eq!(
        c["outputDataConfig"]["s3OutputDataConfig"]["s3Uri"],
        "s3://my-out/results/"
    );
    assert_eq!(c["inputDataConfig"]["s3InputDataConfig"]["s3InputFormat"], "JSONL");
    assert!(
        c["inputDataConfig"]["s3InputDataConfig"]["s3Uri"]
            .as_str()
            .unwrap()
            .starts_with("s3://my-in/jobs/a-graph-")
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
    assert_eq!(statuses, ["InProgress", "InProgress", "Completed"]);
    assert_eq!(*polls.lock().unwrap(), 3);
    assert_eq!(
        events.first(),
        Some(&BatchEvent::Submitted {
            batch_id: "job42".into(),
            requests: 3
        })
    );
    assert_eq!(
        events.last(),
        Some(&BatchEvent::Finished {
            batch_id: "job42".into(),
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
            .contains("bad request")
    );
}
