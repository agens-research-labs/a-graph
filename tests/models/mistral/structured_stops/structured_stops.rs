use super::*;

/// Structured output and stop reasons.
#[tokio::test]
async fn structured_stops() {
    let mut c = call("q");
    c.schema = Some(json!({ "type": "object" }));
    let (r, req) = one_call(c.clone(), Resp::ok(text_answer("{\"a\":1}"))).await;
    assert_eq!(body(&req)["response_format"], json!({ "type": "json_schema", "json_schema": { "name": "output", "schema": { "type": "object" }, "strict": false } }));
    let r = r.unwrap();
    assert_eq!(r.structured, Some(json!({ "a": 1 })));
    assert!(r.message.is_none());

    let (r, _) = one_call(c.clone(), Resp::ok(completion(json!({ "role": "assistant", "content": "{\"a\":" }), "length"))).await;
    assert!(matches!(r, Err(ModelError::Truncated { .. })), "{r:?}");
    let (r, _) = one_call(c, Resp::ok(text_answer("not json"))).await;
    assert!(matches!(r, Err(ModelError::InvalidResponse { .. })), "{r:?}");

    for (finish, expected) in [("length", StopReason::MaxTokens), ("model_length", StopReason::ContextExceeded)] {
        let (r, _) = one_call(call("q"), Resp::ok(completion(json!({ "role": "assistant", "content": "cut" }), finish))).await;
        let r = r.unwrap();
        assert_eq!((r.stop_reason, r.stop_detail.as_deref()), (expected, Some(finish)));
    }
    let (r, _) = one_call(call("q"), Resp::ok(completion(json!({ "role": "assistant", "content": "" }), "error"))).await;
    assert!(matches!(r, Err(ModelError::Provider { .. })), "{r:?}");
}
