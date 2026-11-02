use super::*;

/// A plain call is stateless and maps the answer.
#[tokio::test]
async fn plain_call() {
    let mut c = call("the question");
    c.system = "be brief".into();
    c.history = vec![
        ChatEntry {
            role: Role::User,
            content: "before".into(),
        },
        ChatEntry {
            role: Role::Assistant,
            content: "earlier answer".into(),
        },
    ];
    c.media = vec![
        MediaInput::Markdown("# notes".into()),
        MediaInput::Image {
            format: "jpg".into(),
            data: "QUJD".into(),
        },
    ];
    c.max_tokens = Some(100);
    c.temperature = Some(0.2);
    c.top_p = Some(0.9);
    let (r, req) = one_call(c, Resp::ok(text_answer("hello"))).await;
    let r = r.unwrap();

    assert_eq!(
        (req.method.as_str(), req.path.as_str()),
        ("POST", "/v1beta/interactions")
    );
    assert!(req.headers.contains("x-goog-api-key: test-key"), "{}", req.headers);
    let b = body(&req);
    assert_eq!(b["model"], "gemini-3.5-flash-lite");
    assert_eq!(b["store"], false);
    assert_eq!(b["system_instruction"], "be brief");
    assert_eq!(
        b["generation_config"],
        json!({ "max_output_tokens": 100, "temperature": 0.2, "top_p": 0.9 })
    );
    assert_eq!(
        b["input"],
        json!([
            { "type": "user_input", "content": [{ "type": "text", "text": "before" }] },
            { "type": "model_output", "content": [{ "type": "text", "text": "earlier answer" }] },
            { "type": "user_input", "content": [
                { "type": "text", "text": "# notes" },
                { "type": "image", "mime_type": "image/jpeg", "data": "QUJD" },
                { "type": "text", "text": "the question" }
            ] }
        ])
    );

    assert_eq!(r.message.as_deref(), Some("hello"));
    assert_eq!(
        (r.stop_reason, r.stop_detail.as_deref()),
        (StopReason::End, Some("completed"))
    );
    assert_eq!(r.reasoning, vec![json!({ "type": "thought", "signature": "sig1" })]);
    assert_eq!(r.info.provider, "gemini");
    assert!(r.info.execution_time > 0.0);
    // cached tokens are part of the input; thinking tokens are NOT part of total_output_tokens
    assert_eq!(
        r.info.usage,
        Usage {
            input_tokens: 8,
            cache_read_tokens: 5,
            cache_write_tokens: 0,
            output_tokens: 608,
            reasoning_tokens: 477
        }
    );
}
