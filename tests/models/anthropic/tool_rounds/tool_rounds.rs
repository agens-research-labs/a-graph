use super::*;

/// Tool rounds are echoed and the tool calls come back.
#[tokio::test]
async fn tool_rounds() {
    let answer = message(
        json!([
            { "type": "thinking", "thinking": "let me look", "signature": "sig" },
            { "type": "text", "text": "I will check." },
            { "type": "tool_use", "id": "toolu_1", "name": "get_weather", "input": { "city": "Rome" } }
        ]),
        "tool_use",
    );
    let (r, _) = one_call(call("weather in Rome?"), Resp::ok(answer)).await;
    let r = r.unwrap();
    assert_eq!(r.stop_reason, StopReason::ToolCall);
    assert_eq!(
        r.tools_call,
        vec![ToolCall {
            call_id: "toolu_1".into(),
            function_name: "get_weather".into(),
            arguments: json!({ "city": "Rome" }),
            opaque: None
        }]
    );
    assert_eq!(
        r.reasoning,
        vec![json!({ "type": "thinking", "thinking": "let me look", "signature": "sig" })]
    );
    assert_eq!(r.message.as_deref(), Some("I will check."));

    // the next call sends the round back: the opaque thinking block first, then text and tool_use
    let mut c = call("weather in Rome?");
    c.tool_rounds = vec![ToolRound {
        message: r.message.clone(),
        reasoning: r.reasoning.clone(),
        calls: r.tools_call.clone(),
        results: vec![
            ToolResult {
                call_id: "toolu_1".into(),
                result: json!("22 C"),
                is_error: false,
            },
            ToolResult {
                call_id: "toolu_2".into(),
                result: json!({ "reason": "boom" }),
                is_error: true,
            },
        ],
    }];
    let (_, req) = one_call(c, Resp::ok(text_answer("done"))).await;
    let m = body(&req)["messages"].clone();
    assert_eq!(m.as_array().unwrap().len(), 3);
    assert_eq!(
        m[1],
        json!({ "role": "assistant", "content": [
            { "type": "thinking", "thinking": "let me look", "signature": "sig" },
            { "type": "text", "text": "I will check." },
            { "type": "tool_use", "id": "toolu_1", "name": "get_weather", "input": { "city": "Rome" } }
        ] })
    );
    assert_eq!(
        m[2],
        json!({ "role": "user", "content": [
            { "type": "tool_result", "tool_use_id": "toolu_1", "content": "22 C", "is_error": false },
            { "type": "tool_result", "tool_use_id": "toolu_2", "content": "{\"reason\":\"boom\"}", "is_error": true }
        ] })
    );
}
