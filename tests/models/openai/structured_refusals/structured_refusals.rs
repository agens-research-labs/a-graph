use super::*;

/// Structured output truncation and refusals.
#[tokio::test]
async fn structured_refusals() {
    let mut c = call("q");
    c.schema = Some(json!({ "type": "object" }));

    let (r, req) = one_call(c.clone(), Resp::ok(text_answer("{\"a\":1}"))).await;
    assert_eq!(
        body(&req)["text"],
        json!({ "format": { "type": "json_schema", "name": "output", "schema": { "type": "object" }, "strict": false } })
    );
    let r = r.unwrap();
    assert_eq!(r.structured, Some(json!({ "a": 1 })));
    assert!(r.message.is_none());

    let cut = json!({ "id": "resp_1", "status": "incomplete", "incomplete_details": { "reason": "max_output_tokens" }, "output": [message("{\"a\":")] }).to_string();
    let (r, _) = one_call(c.clone(), Resp::ok(cut.clone())).await;
    assert!(matches!(r, Err(ModelError::Truncated { .. })), "{r:?}");
    // without a schema it is a stop reason, not an error
    let (r, _) = one_call(call("q"), Resp::ok(cut)).await;
    let r = r.unwrap();
    assert_eq!(
        (r.stop_reason, r.stop_detail.as_deref()),
        (StopReason::MaxTokens, Some("incomplete: max_output_tokens"))
    );

    let filtered = json!({ "id": "resp_1", "status": "incomplete", "incomplete_details": { "reason": "content_filter" }, "output": [] }).to_string();
    let (r, _) = one_call(call("q"), Resp::ok(filtered)).await;
    assert_eq!(r.unwrap().stop_reason, StopReason::Refused);

    let refusal = response(
        "completed",
        json!([{ "type": "message", "role": "assistant", "content": [{ "type": "refusal", "refusal": "I cannot help with that." }] }]),
    );
    let (r, _) = one_call(c, Resp::ok(refusal)).await;
    let r = r.unwrap();
    assert_eq!(
        (r.stop_reason, r.message.as_deref()),
        (StopReason::Refused, Some("I cannot help with that."))
    );
    assert!(r.structured.is_none());

    let (r, _) = one_call(
        call("q"),
        Resp::ok(
            json!({ "status": "failed", "error": { "code": "server_error", "message": "it broke" }, "output": [] })
                .to_string(),
        ),
    )
    .await;
    assert!(
        matches!(&r, Err(ModelError::Provider { message, .. }) if message == "it broke"),
        "{r:?}"
    );
}
