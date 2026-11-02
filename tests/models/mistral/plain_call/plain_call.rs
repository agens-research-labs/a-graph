use super::*;

/// A plain call and its answer.
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
            content: "earlier".into(),
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
        ("POST", "/v1/chat/completions")
    );
    assert!(
        req.headers.contains("authorization: bearer test-key"),
        "{}",
        req.headers
    );
    let b = body(&req);
    assert_eq!(
        (
            b["model"].as_str(),
            b["max_tokens"].as_u64(),
            b["temperature"].as_f64(),
            b["top_p"].as_f64()
        ),
        (Some("mistral-small-latest"), Some(100), Some(0.2), Some(0.9))
    );
    assert_eq!(
        b["messages"],
        json!([
            { "role": "system", "content": "be brief" },
            { "role": "user", "content": "before" },
            { "role": "assistant", "content": "earlier" },
            { "role": "user", "content": [
                { "type": "text", "text": "# notes" },
                { "type": "image_url", "image_url": { "url": "data:image/jpeg;base64,QUJD" } },
                { "type": "text", "text": "the question" }
            ] }
        ])
    );
    assert_eq!(r.message.as_deref(), Some("hello"));
    assert_eq!(
        (r.stop_reason, r.stop_detail.as_deref()),
        (StopReason::End, Some("stop"))
    );
    assert_eq!(
        (r.info.provider.as_str(), r.info.model.as_str()),
        ("mistral", "mistral-small-2506")
    );
    assert_eq!(
        r.info.usage,
        Usage {
            input_tokens: 30,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
            output_tokens: 9,
            reasoning_tokens: 4
        }
    );
}
