use super::*;

/// A refusal asks for nothing: its tool calls are dropped, even malformed ones, schema or not. The refusal is a
/// part of a message, before or after the call, or the content filter.
#[tokio::test]
async fn refused_tool_call() {
    let refusal = json!({ "id": "msg_1", "type": "message", "role": "assistant", "status": "completed",
                          "content": [{ "type": "refusal", "refusal": "I cannot help." }] });
    for arguments in ["{\"city\":\"Rome\"}", "{\"city\":"] {
        let fc = json!({ "id": "fc_1", "type": "function_call", "call_id": "call_1", "name": "get_weather", "arguments": arguments, "status": "completed" });
        let filtered = {
            let mut a: Value = serde_json::from_str(&response("incomplete", json!([fc.clone()]))).unwrap();
            a["incomplete_details"] = json!({ "reason": "content_filter" });
            a.to_string()
        };
        let answers = [
            response("completed", json!([refusal.clone(), fc.clone()])),
            response("completed", json!([fc.clone(), refusal.clone()])),
            filtered,
        ];
        for (n, answer) in answers.into_iter().enumerate() {
            for schema in [None, Some(json!({ "type": "object" }))] {
                let mut c = call("weather in Rome?");
                c.schema = schema.clone();
                let (r, _) = one_call(c, Resp::ok(answer.clone())).await;
                let r = r.unwrap_or_else(|e| panic!("{arguments} {n} {schema:?}: {e}"));
                assert_eq!(r.stop_reason, StopReason::Refused, "{arguments} {n} {schema:?}");
                assert!(r.tools_call.is_empty(), "{arguments} {n} {schema:?}");
            }
        }
    }
}
