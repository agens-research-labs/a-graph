use super::*;

/// A refusal asks for nothing: its tool calls are dropped, even malformed ones, schema or not.
#[tokio::test]
async fn refused_tool_call() {
    for stop in ["guardrail_intervened", "content_filtered"] {
        for input in [json!({ "city": "Rome" }), json!("not an object")] {
            for schema in [None, Some(json!({ "type": "object" }))] {
                let mut c = call("weather?");
                c.schema = schema.clone();
                let answer = converse(
                    json!([
                        { "text": "Blocked." },
                        { "toolUse": { "toolUseId": "tooluse_1", "name": "get_weather", "input": input } }
                    ]),
                    stop,
                );
                let (r, _) = one_call(c, Resp::ok(answer)).await;
                let r = r.unwrap_or_else(|e| panic!("{stop} {input} {schema:?}: {e}"));
                assert_eq!(r.stop_reason, StopReason::Refused, "{stop} {input} {schema:?}");
                assert!(r.tools_call.is_empty(), "{stop} {input} {schema:?}");
            }
        }
    }
}
