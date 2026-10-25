use super::*;

/// Structured output and truncation.
#[tokio::test]
async fn structured_truncation() {
    let mut c = call("q");
    c.schema = Some(json!({ "type": "object" }));

    let (r, req) = one_call(c.clone(), Resp::ok(text_answer("{\"a\":1}"))).await;
    assert_eq!(body(&req)["format"], json!({ "type": "object" }));
    let r = r.unwrap();
    assert_eq!(r.structured, Some(json!({ "a": 1 })));
    assert!(r.message.is_none());

    let (r, _) = one_call(c.clone(), Resp::ok(chat_answer(json!({ "role": "assistant", "content": "{\"a\":" }), "length"))).await;
    assert!(matches!(r, Err(ModelError::Truncated { .. })), "{r:?}");

    let (r, _) = one_call(c, Resp::ok(text_answer("not json"))).await;
    assert!(matches!(r, Err(ModelError::InvalidResponse { .. })), "{r:?}");

    let (r, _) = one_call(call("q"), Resp::ok(chat_answer(json!({ "role": "assistant", "content": "cut" }), "length"))).await;
    assert_eq!(r.unwrap().stop_reason, StopReason::MaxTokens);
}
