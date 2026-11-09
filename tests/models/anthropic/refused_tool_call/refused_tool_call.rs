use super::*;

/// A refusal asks for nothing: its tool calls are dropped, even malformed ones, schema or not.
#[tokio::test]
async fn refused_tool_call() {
    for input in [json!({ "city": "Rome" }), json!("not an object")] {
        for schema in [None, Some(json!({ "type": "object" }))] {
            let mut c = call("weather in Rome?");
            c.schema = schema.clone();
            let answer = message(
                json!([
                    { "type": "text", "text": "I cannot help." },
                    { "type": "tool_use", "id": "toolu_1", "name": "get_weather", "input": input }
                ]),
                "refusal",
            );
            let (r, _) = one_call(c, Resp::ok(answer)).await;
            let r = r.unwrap_or_else(|e| panic!("{input} {schema:?}: {e}"));
            assert_eq!(r.stop_reason, StopReason::Refused, "{input} {schema:?}");
            assert!(r.tools_call.is_empty(), "{input} {schema:?}");
        }
    }
}
