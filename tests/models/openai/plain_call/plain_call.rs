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

    assert_eq!((req.method.as_str(), req.path.as_str()), ("POST", "/v1/responses"));
    assert!(
        req.headers.contains("authorization: bearer test-key"),
        "{}",
        req.headers
    );
    let b = body(&req);
    assert_eq!(
        (b["model"].as_str(), b["store"].as_bool(), b["instructions"].as_str()),
        (Some("gpt-5"), Some(false), Some("be brief"))
    );
    assert_eq!(
        (
            b["max_output_tokens"].as_u64(),
            b["temperature"].as_f64(),
            b["top_p"].as_f64()
        ),
        (Some(100), Some(0.2), Some(0.9))
    );
    assert_eq!(
        b["input"],
        json!([
            { "role": "user", "content": "before" },
            { "role": "assistant", "content": "earlier" },
            { "role": "user", "content": [
                { "type": "input_text", "text": "# notes" },
                { "type": "input_image", "image_url": "data:image/jpeg;base64,QUJD", "detail": "auto" },
                { "type": "input_text", "text": "the question" }
            ] }
        ])
    );
    assert!(
        b.get("reasoning").is_none()
            && b.get("include").is_none()
            && b.get("tools").is_none()
            && b.get("text").is_none()
    );

    assert_eq!(r.message.as_deref(), Some("hello"));
    assert_eq!(
        (r.stop_reason, r.stop_detail.as_deref()),
        (StopReason::End, Some("completed"))
    );
    assert_eq!(
        (r.info.provider.as_str(), r.info.model.as_str()),
        ("openai", "gpt-5-2026")
    );
    assert!(r.info.execution_time > 0.0);
    // cached tokens are part of input_tokens; reasoning tokens are part of output_tokens
    assert_eq!(
        r.info.usage,
        Usage {
            input_tokens: 60,
            cache_read_tokens: 40,
            cache_write_tokens: 3,
            output_tokens: 50,
            reasoning_tokens: 20
        }
    );
}
