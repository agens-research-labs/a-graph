use super::*;

/// Live call schema and tool.
#[tokio::test]
#[ignore = "needs a local Ollama with llama3.2"]
async fn live_call() {
    let model = live("llama3.2:latest");
    let mut c = call("Reply with exactly: ok");
    c.max_tokens = Some(20);
    let r = model.invoke(&c).await.unwrap();
    assert!(r.message.unwrap().to_lowercase().contains("ok"));
    assert!(r.info.usage.input_tokens > 0 && r.info.usage.output_tokens > 0);

    let mut c = call("The capital of Italy.");
    c.max_tokens = Some(100);
    c.schema = Some(json!({ "type": "object", "properties": { "city": { "type": "string" } }, "required": ["city"] }));
    // the provider gives the text; a-graph reads the JSON of it
    let r = model.invoke(&c).await.unwrap();
    match a_graph::models::structured::check(c.schema.as_ref().unwrap(), &r) {
        a_graph::models::structured::Checked::Valid(v) => assert_eq!(v["city"], "Rome"),
        other => panic!("{other:?}"),
    }

    let mut c = call("What is the weather in Rome? Use the tool.");
    c.tools = vec![ToolDefinition {
        name: "get_weather".into(),
        description: "Current weather of a city".into(),
        parameters: json!({ "type": "object", "properties": { "city": { "type": "string" } }, "required": ["city"] }),
    }];
    let first = model.invoke(&c).await.unwrap();
    assert_eq!(first.stop_reason, StopReason::ToolCall);
    c.tool_rounds = vec![ToolRound {
        message: first.message.clone(),
        reasoning: first.reasoning.clone(),
        calls: first.tools_call.clone(),
        results: vec![ToolResult {
            call_id: first.tools_call[0].call_id.clone(),
            result: json!("sunny, 22 C"),
            is_error: false,
        }],
        corrections: vec![],
    }];
    let second = model.invoke(&c).await.unwrap();
    assert_eq!(second.stop_reason, StopReason::End);
    assert!(second.message.unwrap().contains("22"));
}
