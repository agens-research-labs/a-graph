use super::*;

/// Structured output and truncation.
#[tokio::test]
async fn structured_truncation() {
    let mut c = call("q");
    c.schema = Some(json!({ "type": "object" }));

    let (r, req) = one_call(c.clone(), Resp::ok(text_answer("{\"a\":1}"))).await;
    assert_eq!(
        body(&req)["response_format"],
        json!({ "type": "text", "mime_type": "application/json", "schema": { "type": "object" } })
    );
    let r = r.unwrap();
    assert_eq!(r.structured, Some(json!({ "a": 1 })));
    assert!(r.message.is_none());

    let (r, _) = one_call(
        c.clone(),
        Resp::ok(interaction("incomplete", json!([output("{\"a\":")]))),
    )
    .await;
    assert!(matches!(r, Err(ModelError::Truncated { .. })), "{r:?}");

    let (r, _) = one_call(c, Resp::ok(text_answer("not json"))).await;
    assert!(matches!(r, Err(ModelError::InvalidResponse { .. })), "{r:?}");

    // without a schema, running out of tokens is a stop reason, not an error
    let (r, _) = one_call(call("q"), Resp::ok(interaction("incomplete", json!([output("cut")])))).await;
    let r = r.unwrap();
    assert_eq!(
        (r.stop_reason, r.stop_detail.as_deref()),
        (StopReason::MaxTokens, Some("incomplete"))
    );
}
