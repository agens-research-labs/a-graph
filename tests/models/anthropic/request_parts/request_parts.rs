use super::*;

/// The request carries every part of the invocation.
#[tokio::test]
async fn request_parts() {
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
    c.tools = vec![ToolDefinition {
        name: "get_weather".into(),
        description: "weather".into(),
        parameters: json!({ "type": "object" }),
    }];
    c.temperature = Some(0.2);
    c.top_p = Some(0.9);
    c.reasoning = Some(Reasoning::Minimal);
    c.schema = Some(json!({ "type": "object" }));
    let (r, req) = one_call(c, Resp::ok(text_answer("{}"))).await;
    r.unwrap();
    let b = body(&req);

    assert_eq!(b["model"], "claude-haiku-4-5");
    assert_eq!(b["max_tokens"], 100);
    assert_eq!(b["system"], "be brief");
    assert_eq!(b["temperature"], 0.2);
    assert_eq!(b["top_p"], 0.9);
    assert_eq!(
        b["tools"][0],
        json!({ "name": "get_weather", "description": "weather", "input_schema": { "type": "object" } })
    );
    // minimal has no level of its own: it goes to low; summarized so the thinking text is returned
    assert_eq!(b["thinking"], json!({ "type": "adaptive", "display": "summarized" }));
    assert_eq!(
        b["output_config"],
        json!({ "effort": "low", "format": { "type": "json_schema", "schema": { "type": "object" } } })
    );

    let m = b["messages"].as_array().unwrap();
    assert_eq!(m.len(), 3);
    assert_eq!(m[0], json!({ "role": "user", "content": "before" }));
    assert_eq!(m[1], json!({ "role": "assistant", "content": "earlier answer" }));
    // media first (the jpg is sent as image/jpeg), then the question
    assert_eq!(
        m[2]["content"],
        json!([
            { "type": "text", "text": "# notes" },
            { "type": "image", "source": { "type": "base64", "media_type": "image/jpeg", "data": "QUJD" } },
            { "type": "text", "text": "the question" }
        ])
    );
}
