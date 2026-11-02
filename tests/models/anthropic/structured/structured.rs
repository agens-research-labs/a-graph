use super::*;

/// Structured output.
#[tokio::test]
async fn structured() {
    let mut c = call("q");
    c.schema = Some(json!({ "type": "object" }));

    let (r, _) = one_call(c.clone(), Resp::ok(text_answer("{\"a\":1}"))).await;
    let r = r.unwrap();
    assert_eq!(r.structured, Some(json!({ "a": 1 })));
    assert!(r.message.is_none());

    let (r, _) = one_call(
        c.clone(),
        Resp::ok(message(json!([{ "type": "text", "text": "{\"a\":" }]), "max_tokens")),
    )
    .await;
    assert!(matches!(r, Err(ModelError::Truncated { .. })), "{r:?}");

    let (r, _) = one_call(c.clone(), Resp::ok(text_answer("not json"))).await;
    assert!(matches!(r, Err(ModelError::InvalidResponse { .. })), "{r:?}");

    // without a schema, max_tokens is a stop reason, not an error
    let (r, _) = one_call(
        call("q"),
        Resp::ok(message(json!([{ "type": "text", "text": "cut" }]), "max_tokens")),
    )
    .await;
    assert_eq!(r.unwrap().stop_reason, StopReason::MaxTokens);
}
