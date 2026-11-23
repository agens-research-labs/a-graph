use super::*;

/// Tool calls keep their item and the round goes back.
#[tokio::test]
async fn tool_item() {
    let fc = json!({ "id": "fc_1", "type": "function_call", "call_id": "call_1", "name": "get_weather", "arguments": "{\"city\":\"Rome\"}", "status": "completed" });
    let rs = json!({ "id": "rs_1", "type": "reasoning", "summary": [{ "type": "summary_text", "text": "thinking" }], "encrypted_content": "enc" });
    let (r, _) = one_call(
        call("weather?"),
        Resp::ok(response("completed", json!([rs, message("Checking."), fc]))),
    )
    .await;
    let r = r.unwrap();
    assert_eq!(r.stop_reason, StopReason::ToolCall);
    assert_eq!(r.tools_call.len(), 1);
    assert_eq!(
        (r.tools_call[0].call_id.as_str(), r.tools_call[0].function_name.as_str()),
        ("call_1", "get_weather")
    );
    assert_eq!(r.tools_call[0].arguments, json!({ "city": "Rome" }));
    assert_eq!(r.tools_call[0].opaque.as_ref().unwrap()["id"], "fc_1");
    assert_eq!(r.reasoning, vec![rs.clone()]);
    assert_eq!(r.message.as_deref(), Some("Checking."));

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
        json!([{ "type": "function", "name": "get_weather", "description": "weather", "parameters": { "type": "object" }, "strict": false }])
    );
    assert_eq!(b["include"], json!(["reasoning.encrypted_content"]));
    let input = b["input"].as_array().unwrap();
    assert_eq!(input.len(), 1 + 4);
    assert_eq!(input[1], rs);
    assert_eq!(input[2], json!({ "role": "assistant", "content": "Checking." }));
    assert_eq!(input[3], fc);
    assert_eq!(
        input[4],
        json!({ "type": "function_call_output", "call_id": "call_1", "output": "error: {\"reason\":\"down\"}" })
    );
}
