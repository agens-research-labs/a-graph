use super::*;

/// A plain call goes to converse signed and maps the answer.
#[tokio::test]
async fn plain_call() {
    let mut c = call("the question");
    c.system = "be brief".into();
    c.history = vec![ChatEntry { role: Role::User, content: "before".into() }, ChatEntry { role: Role::Assistant, content: "earlier".into() }];
    c.media = vec![MediaInput::Markdown("# notes".into()), MediaInput::Image { format: "jpg".into(), data: STANDARD.encode("IMG") }];
    c.max_tokens = Some(100);
    c.temperature = Some(0.5);
    c.top_p = Some(0.75);
    let (r, req) = one_call(c, Resp::ok(text_answer("hello"))).await;
    let r = r.unwrap();

    assert_eq!(req.method, "POST");
    assert!(req.path.starts_with("/model/anthropic.claude-sonnet-4-5/converse"), "{}", req.path);
    // the request is signed with the keys of the environment, for the region of the yml
    assert!(req.headers.contains("authorization: aws4-hmac-sha256 credential=akiatest/") && req.headers.contains("/eu-west-1/bedrock/aws4_request"), "{}", req.headers);
    assert!(!req.headers.contains("secret-test-key"));

    let b = body(&req);
    assert_eq!(b["system"], json!([{ "text": "be brief" }]));
    assert_eq!(b["inferenceConfig"], json!({ "maxTokens": 100, "temperature": 0.5, "topP": 0.75 }));
    let m = b["messages"].as_array().unwrap();
    assert_eq!(m.len(), 3);
    assert_eq!(m[0], json!({ "role": "user", "content": [{ "text": "before" }] }));
    assert_eq!(m[1], json!({ "role": "assistant", "content": [{ "text": "earlier" }] }));
    assert_eq!(
        m[2],
        json!({ "role": "user", "content": [
            { "text": "# notes" },
            { "image": { "format": "jpeg", "source": { "bytes": STANDARD.encode("IMG") } } },
            { "text": "the question" }
        ] })
    );
    assert!(b.get("toolConfig").is_none() && b.get("outputConfig").is_none() && b.get("additionalModelRequestFields").is_none());

    assert_eq!(r.message.as_deref(), Some("hello"));
    assert_eq!((r.stop_reason, r.stop_detail.as_deref()), (StopReason::End, Some("end_turn")));
    assert_eq!(r.info.provider, "bedrock");
    assert!(r.info.execution_time > 0.0);
    // the cached tokens are counted apart from the input tokens
    assert_eq!(r.info.usage, Usage { input_tokens: 30, cache_read_tokens: 5, cache_write_tokens: 2, output_tokens: 9, reasoning_tokens: 0 });
}
