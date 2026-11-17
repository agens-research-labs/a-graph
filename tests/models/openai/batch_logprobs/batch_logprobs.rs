use super::*;

/// In a batch a request asks the logprobs as a single call does, and its result gives them.
#[tokio::test]
async fn batch_logprobs() {
    let answer = json!({ "id": "resp_1", "object": "response", "status": "completed", "model": "gpt-5", "output": [
        { "id": "msg_1", "type": "message", "role": "assistant", "status": "completed", "content": [
            { "type": "output_text", "text": "hi", "annotations": [], "logprobs": [{ "token": "hi", "logprob": -0.5, "bytes": [104, 105], "top_logprobs": [] }] }
        ] }
    ] });
    let line =
        json!({ "custom_id": "one", "response": { "status_code": 200, "body": answer }, "error": null }).to_string();
    let handler = move |req: &Req| match (req.method.as_str(), req.path.as_str()) {
        ("POST", "/v1/files") => Resp::ok(json!({ "id": "file-in" }).to_string()),
        ("POST", "/v1/batches") => Resp::ok(json!({ "id": "batch_1", "status": "validating" }).to_string()),
        ("GET", "/v1/batches/batch_1") => Resp::ok(
            json!({ "id": "batch_1", "status": "completed", "output_file_id": "file-out", "error_file_id": null })
                .to_string(),
        ),
        ("GET", "/v1/files/file-out/content") => Resp::ok(line.clone()),
        _ => Resp::status(404, "{}"),
    };
    let (base, seen) = serve(handler).await;
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut invocation = call("q");
    invocation.logprobs = Some(1);
    let requests = vec![BatchRequest {
        id: "one".into(),
        invocation,
    }];

    let results = client(&base).invoke_batch(requests, tx).await.unwrap();

    let upload = seen.lock().unwrap()[0].clone();
    let sent: Value = upload
        .body
        .lines()
        .find(|l| l.starts_with('{'))
        .map(|l| serde_json::from_str(l).unwrap())
        .unwrap();
    assert_eq!(sent["body"]["include"], json!(["message.output_text.logprobs"]));
    assert_eq!(sent["body"]["top_logprobs"], json!(1));
    let r = results[0].result.as_ref().unwrap();
    assert_eq!(
        r.logprobs,
        vec![vec![TokenLogprob {
            token: "hi".into(),
            logprob: -0.5,
            bytes: Some(b"hi".to_vec()),
            top_logprobs: vec![],
        }]]
    );
}
