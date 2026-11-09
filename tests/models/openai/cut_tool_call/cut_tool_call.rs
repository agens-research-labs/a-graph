use super::*;

/// A cut answer that asks for a tool is an error, schema or not: the call may be cut and is not run.
#[tokio::test]
async fn cut_tool_call() {
    for schema in [None, Some(json!({ "type": "object" }))] {
        let mut c = call("weather in Rome?");
        c.schema = schema.clone();
        let mut answer: Value = serde_json::from_str(&response(
            "incomplete",
            json!([
                { "id": "fc_1", "type": "function_call", "call_id": "call_1", "name": "get_weather", "arguments": "{\"city\":\"Ro\"}", "status": "incomplete" }
            ]),
        ))
        .unwrap();
        answer["incomplete_details"] = json!({ "reason": "max_output_tokens" });
        let (r, _) = one_call(c, Resp::ok(answer.to_string())).await;
        assert!(matches!(r, Err(ModelError::Truncated { .. })), "{schema:?}: {r:?}");
    }
}
