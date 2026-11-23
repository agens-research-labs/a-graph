use super::*;

/// The corrections of `schema_prompt` come after the tool rounds: the answer that was not usable JSON as the
/// assistant, then what a-graph asked instead as the user, last of all.
#[tokio::test]
async fn correction_last() {
    let mut c = call("weather?");
    c.tools = vec![ToolDefinition {
        name: "get_weather".into(),
        description: "weather".into(),
        parameters: json!({ "type": "object", "properties": { "city": { "type": "string" } } }),
    }];
    c.tool_rounds = vec![ToolRound {
        message: None,
        reasoning: vec![],
        calls: vec![ToolCall {
            call_id: "call_1".into(),
            function_name: "get_weather".into(),
            arguments: json!({ "city": "Rome" }),
            opaque: None,
        }],
        results: vec![ToolResult {
            call_id: "call_1".into(),
            result: json!("sunny"),
            is_error: false,
        }],
        corrections: vec![],
    }];
    c.corrections = vec![
        ChatEntry {
            role: Role::Assistant,
            content: "six".into(),
        },
        ChatEntry {
            role: Role::User,
            content: "answer with JSON".into(),
        },
    ];
    let (_, req) = one_call(c, Resp::ok(text_answer("{}"))).await;
    let sent = body(&req)["messages"].as_array().unwrap().clone();
    assert_eq!(
        sent[sent.len() - 2..],
        [
            json!({ "role": "assistant", "content": "six" }),
            json!({ "role": "user", "content": "answer with JSON" })
        ],
        "{sent:?}"
    );
}
