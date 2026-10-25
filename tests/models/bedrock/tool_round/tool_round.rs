use super::*;

/// Tool calls and the round that goes back.
#[tokio::test]
async fn tool_round() {
    let answer = converse(
        json!([
            { "reasoningContent": { "reasoningText": { "text": "let me look", "signature": "sig" } } },
            { "reasoningContent": { "redactedContent": STANDARD.encode("secret") } },
            { "text": "Checking." },
            { "toolUse": { "toolUseId": "tooluse_1", "name": "get_weather", "input": { "city": "Rome", "days": 3 } } }
        ]),
        "tool_use",
    );
    let (r, _) = one_call(call("weather?"), Resp::ok(answer)).await;
    let r = r.unwrap();
    assert_eq!(r.stop_reason, StopReason::ToolCall);
    assert_eq!(r.tools_call, vec![ToolCall { call_id: "tooluse_1".into(), function_name: "get_weather".into(), arguments: json!({ "city": "Rome", "days": 3 }), opaque: None }]);
    assert_eq!(r.reasoning, vec![json!({ "text": "let me look", "signature": "sig" }), json!({ "redacted": STANDARD.encode("secret") })]);
    assert_eq!(r.message.as_deref(), Some("Checking."));

    let mut c = call("weather?");
    c.tools = vec![ToolDefinition { name: "get_weather".into(), description: "weather".into(), parameters: json!({ "type": "object", "properties": { "city": { "type": "string" } } }) }];
    c.tool_rounds = vec![ToolRound {
        message: r.message.clone(),
        reasoning: r.reasoning.clone(),
        calls: r.tools_call.clone(),
        results: vec![ToolResult { call_id: "tooluse_1".into(), result: json!({ "reason": "down" }), is_error: true }],
    }];
    let (_, req) = one_call(c, Resp::ok(text_answer("done"))).await;
    let b = body(&req);
    assert_eq!(b["toolConfig"]["tools"][0]["toolSpec"]["name"], "get_weather");
    assert_eq!(b["toolConfig"]["tools"][0]["toolSpec"]["inputSchema"], json!({ "json": { "type": "object", "properties": { "city": { "type": "string" } } } }));
    let m = b["messages"].as_array().unwrap();
    assert_eq!(m.len(), 3);
    assert_eq!(
        m[1],
        json!({ "role": "assistant", "content": [
            { "reasoningContent": { "reasoningText": { "text": "let me look", "signature": "sig" } } },
            { "reasoningContent": { "redactedContent": STANDARD.encode("secret") } },
            { "text": "Checking." },
            { "toolUse": { "toolUseId": "tooluse_1", "name": "get_weather", "input": { "city": "Rome", "days": 3 } } }
        ] })
    );
    assert_eq!(m[2], json!({ "role": "user", "content": [{ "toolResult": { "toolUseId": "tooluse_1", "content": [{ "text": "{\"reason\":\"down\"}" }], "status": "error" } }] }));
}
