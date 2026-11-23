use super::*;

/// Tool calls keep their signature and the round goes back.
#[tokio::test]
async fn tool_signature() {
    let answer = interaction(
        "requires_action",
        json!([
            { "type": "thought", "signature": "thought-sig", "summary": [{ "text": "thinking about weather" }] },
            output("Let me check."),
            { "type": "function_call", "id": "call_1", "signature": "call-sig", "name": "get_weather", "arguments": { "city": "Rome" } }
        ]),
    );
    let (r, _) = one_call(call("weather?"), Resp::ok(answer)).await;
    let r = r.unwrap();
    assert_eq!(r.stop_reason, StopReason::ToolCall);
    assert_eq!(
        r.tools_call,
        vec![ToolCall {
            call_id: "call_1".into(),
            function_name: "get_weather".into(),
            arguments: json!({ "city": "Rome" }),
            opaque: Some(json!("call-sig"))
        }]
    );
    assert_eq!(r.reasoning[0]["signature"], "thought-sig");
    assert_eq!(r.message.as_deref(), Some("Let me check."));

    let mut c = call("weather?");
    c.tools = vec![ToolDefinition {
        name: "get_weather".into(),
        description: "weather".into(),
        parameters: json!({ "type": "object" }),
    }];
    c.tool_rounds = vec![ToolRound {
        message: r.message.clone(),
        reasoning: r.reasoning.clone(),
        calls: r.tools_call.clone(),
        results: vec![ToolResult {
            call_id: "call_1".into(),
            result: json!({ "reason": "down" }),
            is_error: true,
        }],
        corrections: vec![],
    }];
    let (_, req) = one_call(c, Resp::ok(text_answer("done"))).await;
    let b = body(&req);
    assert_eq!(
        b["tools"],
        json!([{ "type": "function", "name": "get_weather", "description": "weather", "parameters": { "type": "object" } }])
    );
    let input = b["input"].as_array().unwrap();
    assert_eq!(input.len(), 1 + 4);
    // thought steps first (opaque), then the text, the call with its signature, the result
    assert_eq!(input[1], r.reasoning[0]);
    assert_eq!(input[2], output("Let me check."));
    assert_eq!(
        input[3],
        json!({ "type": "function_call", "id": "call_1", "name": "get_weather", "arguments": { "city": "Rome" }, "signature": "call-sig" })
    );
    assert_eq!(
        input[4],
        json!({ "type": "function_result", "name": "get_weather", "call_id": "call_1", "result": [{ "type": "text", "text": "{\"reason\":\"down\"}" }], "is_error": true })
    );
}
