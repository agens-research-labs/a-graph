use super::*;

/// Live tool round trip.
#[tokio::test]
#[ignore = "calls the real API with GEMINI_API_KEY"]
async fn live_tool_round() {
    let model = live("gemini-3.5-flash-lite");
    let mut c = call("What is the weather in Rome? Use the tool.");
    c.max_tokens = Some(300);
    c.tools = vec![ToolDefinition {
        name: "get_weather".into(),
        description: "Current weather of a city".into(),
        parameters: json!({ "type": "object", "properties": { "city": { "type": "string" } }, "required": ["city"] }),
    }];
    let first = model.invoke(&c).await.unwrap();
    assert_eq!(first.stop_reason, StopReason::ToolCall);
    let asked = first.tools_call[0].clone();
    assert_eq!(asked.function_name, "get_weather");

    c.tool_rounds = vec![ToolRound {
        message: first.message.clone(),
        reasoning: first.reasoning.clone(),
        calls: first.tools_call.clone(),
        results: vec![ToolResult { call_id: asked.call_id, result: json!("sunny, 22 C"), is_error: false }],
    }];
    let second = model.invoke(&c).await.unwrap();
    assert_eq!(second.stop_reason, StopReason::End);
    assert!(second.message.unwrap().contains("22"));
}
