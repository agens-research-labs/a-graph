use super::*;

/// A cut answer that asks for a tool is an error, schema or not: the call may be cut and is not run.
#[tokio::test]
async fn cut_tool_call() {
    for stop in ["max_tokens", "model_context_window_exceeded"] {
        for schema in [None, Some(json!({ "type": "object" }))] {
            let mut c = call("weather?");
            c.schema = schema.clone();
            let answer = converse(
                json!([
                    { "text": "Checking." },
                    { "toolUse": { "toolUseId": "tooluse_1", "name": "get_weather", "input": { "city": "Ro" } } }
                ]),
                stop,
            );
            let (r, _) = one_call(c, Resp::ok(answer)).await;
            assert!(
                matches!(r, Err(ModelError::Truncated { .. })),
                "{stop} {schema:?}: {r:?}"
            );
        }
    }
}
