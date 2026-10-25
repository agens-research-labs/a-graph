use super::*;

/// Thinking comes back and goes with the tool round.
#[tokio::test]
async fn thinking_tool_round() {
    let answer = chat_answer(
        json!({ "role": "assistant", "content": "", "thinking": "I should ask the tool", "tool_calls": [
            { "id": "call_abc", "function": { "index": 0, "name": "get_weather", "arguments": { "city": "Rome" } } },
            { "function": { "name": "get_time", "arguments": { "tz": "CET" } } }
        ] }),
        "stop",
    );
    let (r, _) = one_call(call("weather?"), Resp::ok(answer)).await;
    let r = r.unwrap();
    // done_reason is `stop` even when tools are asked: the tool calls decide
    assert_eq!(r.stop_reason, StopReason::ToolCall);
    assert_eq!(r.tools_call.len(), 2);
    assert_eq!(r.tools_call[0], ToolCall { call_id: "call_abc".into(), function_name: "get_weather".into(), arguments: json!({ "city": "Rome" }), opaque: None });
    // no id from the server: one is made
    assert_eq!(r.tools_call[1].call_id, "call_1");
    assert!(r.message.is_none());
    assert_eq!(r.reasoning, vec![json!({ "thinking": "I should ask the tool" })]);

    let mut c = call("weather?");
    c.tools = vec![ToolDefinition { name: "get_weather".into(), description: "weather".into(), parameters: json!({ "type": "object" }) }];
    c.tool_rounds = vec![ToolRound {
        message: None,
        reasoning: r.reasoning.clone(),
        calls: r.tools_call.clone(),
        results: vec![
            ToolResult { call_id: "call_abc".into(), result: json!("sunny, 22 C"), is_error: false },
            ToolResult { call_id: "call_1".into(), result: json!("clock stopped"), is_error: true },
        ],
    }];
    let (_, req) = one_call(c, Resp::ok(text_answer("done"))).await;
    let b = body(&req);
    assert_eq!(b["tools"], json!([{ "type": "function", "function": { "name": "get_weather", "description": "weather", "parameters": { "type": "object" } } }]));
    let m = b["messages"].as_array().unwrap();
    assert_eq!(m.len(), 4);
    assert_eq!(
        m[1],
        json!({ "role": "assistant", "content": "", "thinking": "I should ask the tool", "tool_calls": [
            { "id": "call_abc", "function": { "name": "get_weather", "arguments": { "city": "Rome" } } },
            { "id": "call_1", "function": { "name": "get_time", "arguments": { "tz": "CET" } } }
        ] })
    );
    assert_eq!(m[2], json!({ "role": "tool", "tool_name": "get_weather", "content": "sunny, 22 C" }));
    // the contract has no error flag: the text says it
    assert_eq!(m[3], json!({ "role": "tool", "tool_name": "get_time", "content": "error: clock stopped" }));
}
