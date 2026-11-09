use super::*;

/// A cut answer that asks for a tool is an error, schema or not: the call may be cut and is not run.
#[tokio::test]
async fn cut_tool_call() {
    for schema in [None, Some(json!({ "type": "object" }))] {
        let mut c = call("weather in Rome?");
        c.schema = schema.clone();
        let answer = interaction(
            "incomplete",
            json!([
                output("I will check."),
                { "type": "function_call", "id": "call_1", "name": "get_weather", "arguments": { "city": "Ro" } }
            ]),
        );
        let (r, _) = one_call(c, Resp::ok(answer)).await;
        assert!(matches!(r, Err(ModelError::Truncated { .. })), "{schema:?}: {r:?}");
    }
}
