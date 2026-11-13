use super::*;

/// A batch result `type` that is not a fixed code stays out of the message, as the rest of what
/// the provider wrote; a result with no `type` at all says so, with nothing to show.
#[tokio::test]
async fn batch_type_not_fixed() {
    let long = "a".repeat(65);
    let lines = [
        json!({ "custom_id": "one", "result": { "type": long } }),
        json!({ "custom_id": "two", "result": {} }),
    ]
    .iter()
    .map(|v| v.to_string())
    .collect::<Vec<_>>()
    .join("\n");
    let handler = move |req: &Req| match (req.method.as_str(), req.path.as_str()) {
        ("POST", "/v1/messages/batches") => Resp::ok(json!({ "id": "msgbatch_1" }).to_string()),
        ("GET", "/v1/messages/batches/msgbatch_1") => {
            Resp::ok(json!({ "id": "msgbatch_1", "processing_status": "ended" }).to_string())
        }
        ("GET", "/v1/messages/batches/msgbatch_1/results") => Resp::ok(lines.clone()),
        _ => Resp::status(404, "{}"),
    };
    let (base, _) = serve(handler).await;
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let requests = ["one", "two"]
        .map(|id| BatchRequest {
            id: id.into(),
            invocation: call(id),
        })
        .to_vec();
    let results = client(&base).invoke_batch(requests, tx).await.unwrap();

    let get = |id: &str| {
        results
            .iter()
            .find(|r| r.id == id)
            .unwrap()
            .result
            .as_ref()
            .unwrap_err()
    };

    let one = get("one");
    assert_eq!(
        one.to_string(),
        "model `fast` (anthropic): invalid response: an unknown batch result type",
        "{one:?}"
    );
    assert_eq!(one.said(), Some(long.as_str()));

    let two = get("two");
    assert_eq!(
        two.to_string(),
        "model `fast` (anthropic): invalid response: the batch result has no `type`",
        "{two:?}"
    );
    assert_eq!(two.said(), None);
}
