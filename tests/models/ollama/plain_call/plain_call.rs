use super::*;

/// A plain call and its answer.
#[tokio::test]
async fn plain_call() {
    let mut c = call("the question");
    c.system = "be brief".into();
    c.history = vec![ChatEntry { role: Role::User, content: "before".into() }, ChatEntry { role: Role::Assistant, content: "earlier".into() }];
    c.media = vec![MediaInput::Markdown("# notes".into()), MediaInput::Image { format: "png".into(), data: "QUJD".into() }];
    c.max_tokens = Some(100);
    c.temperature = Some(0.2);
    c.top_p = Some(0.9);
    let (r, req) = one_call(c, Resp::ok(text_answer("hello"))).await;
    let r = r.unwrap();

    assert_eq!((req.method.as_str(), req.path.as_str()), ("POST", "/api/chat"));
    let b = body(&req);
    assert_eq!((b["model"].as_str(), b["stream"].as_bool()), (Some("llama3.2:latest"), Some(false)));
    assert_eq!(b["options"], json!({ "temperature": 0.2, "top_p": 0.9, "num_predict": 100 }));
    assert_eq!(
        b["messages"],
        json!([
            { "role": "system", "content": "be brief" },
            { "role": "user", "content": "before" },
            { "role": "assistant", "content": "earlier" },
            { "role": "user", "content": "# notes\n\nthe question", "images": ["QUJD"] }
        ])
    );
    assert!(b.get("think").is_none() && b.get("tools").is_none() && b.get("format").is_none());

    assert_eq!(r.message.as_deref(), Some("hello"));
    assert_eq!((r.stop_reason, r.stop_detail.as_deref()), (StopReason::End, Some("stop")));
    assert_eq!(r.info.provider, "ollama");
    assert!(r.info.execution_time > 0.0);
    // the prompt count includes the cached tokens
    assert_eq!(r.info.usage, Usage { input_tokens: 23, cache_read_tokens: 10, cache_write_tokens: 0, output_tokens: 7, reasoning_tokens: 0 });
}
