use super::*;

/// Thinking chunks and tool calls go back as they came.
#[tokio::test]
async fn thinking_tools() {
    let thinking = json!({ "type": "thinking", "thinking": [{ "type": "text", "text": "let me check" }], "closed": true });
    let answer = completion(
        json!({ "role": "assistant", "content": [thinking.clone(), { "type": "text", "text": "Checking." }],
                "tool_calls": [{ "id": "abc123xyz", "type": "function", "function": { "name": "get_weather", "arguments": "{\"city\":\"Rome\"}" } }] }),
        "tool_calls",
    );
    let (r, _) = one_call(call("weather?"), Resp::ok(answer)).await;
    let r = r.unwrap();
    assert_eq!(r.stop_reason, StopReason::ToolCall);
    assert_eq!(r.tools_call, vec![ToolCall { call_id: "abc123xyz".into(), function_name: "get_weather".into(), arguments: json!({ "city": "Rome" }), opaque: None }]);
    assert_eq!(r.reasoning, vec![thinking.clone()]);
    assert_eq!(r.message.as_deref(), Some("Checking."));

    let mut c = call("weather?");
    c.tools = vec![ToolDefinition { name: "get_weather".into(), description: "weather".into(), parameters: json!({ "type": "object" }) }];
    c.tool_rounds = vec![ToolRound {
        message: r.message.clone(),
        reasoning: r.reasoning.clone(),
        calls: r.tools_call.clone(),
        results: vec![ToolResult { call_id: "abc123xyz".into(), result: json!({ "reason": "down" }), is_error: true }],
    }];
    let (_, req) = one_call(c, Resp::ok(text_answer("done"))).await;
    let b = body(&req);
    assert_eq!(b["tools"], json!([{ "type": "function", "function": { "name": "get_weather", "description": "weather", "parameters": { "type": "object" } } }]));
    let m = b["messages"].as_array().unwrap();
    assert_eq!(m.len(), 3);
    assert_eq!(
        m[1],
        json!({ "role": "assistant", "content": [thinking, { "type": "text", "text": "Checking." }],
                "tool_calls": [{ "id": "abc123xyz", "type": "function", "function": { "name": "get_weather", "arguments": "{\"city\":\"Rome\"}" } }] })
    );
    assert_eq!(m[2], json!({ "role": "tool", "tool_call_id": "abc123xyz", "name": "get_weather", "content": "error: {\"reason\":\"down\"}" }));
}
