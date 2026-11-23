//! The live check of `schema_prompt` after a tool round: the real API of a provider accepts a correction (the
//! answer that was not JSON, as the assistant, then the request of a-graph, as the user) after a tool call and
//! its result. Nothing is guessed: the model asks for the tool, gets its result, and the call is sent again with
//! the correction at the end.

use a_graph::models::structured::CORRECTION;
use a_graph::models::*;
use serde_json::json;

/// Asks `model` for the tool, answers it, then sends the correction; the API must accept the request.
pub async fn correction_accepted(model: &dyn Llm, mut call: ModelInvocation) {
    call.user = "What is the weather in Rome? Use the tool.".into();
    call.max_tokens = Some(400);
    call.tools = vec![ToolDefinition {
        name: "get_weather".into(),
        description: "Current weather of a city".into(),
        parameters: json!({ "type": "object", "properties": { "city": { "type": "string" } }, "required": ["city"] }),
    }];
    let first = model.invoke(&call).await.unwrap();
    assert_eq!(first.stop_reason, StopReason::ToolCall, "{first:?}");
    let asked = first.tools_call[0].clone();
    call.tool_rounds = vec![ToolRound {
        message: first.message.clone(),
        reasoning: first.reasoning.clone(),
        calls: first.tools_call.clone(),
        results: vec![ToolResult {
            call_id: asked.call_id,
            result: json!("sunny, 22 C"),
            is_error: false,
        }],
        corrections: vec![],
    }];
    call.corrections = vec![
        ChatEntry {
            role: Role::Assistant,
            content: "It is sunny in Rome, 22 C.".into(),
        },
        ChatEntry {
            role: Role::User,
            content: format!(
                "{CORRECTION} The schema is {{\"type\":\"object\",\"properties\":{{\"celsius\":{{\"type\":\"number\"}}}}}}."
            ),
        },
    ];
    let again = model.invoke(&call).await;
    assert!(again.is_ok(), "the API refused the correction: {again:?}");
}
