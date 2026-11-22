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
    // the text comes back as it came: the JSON is read by `models::structured`, whatever the provider
    let r = r.unwrap();
    assert_eq!((r.message.as_deref(), &r.structured), (Some("{\"a\":1}"), &None));

    let cut = json!({ "id": "resp_1", "status": "incomplete", "incomplete_details": { "reason": "max_output_tokens" }, "output": [message("{\"a\":")] }).to_string();
    let (r, _) = one_call(c.clone(), Resp::ok(cut.clone())).await;
    assert!(
        r.as_ref()
            .is_ok_and(|r| r.stop_reason.is_cut() && r.message.as_deref() == Some("{\"a\":")),
        "{r:?}"
    );
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
        matches!(&r, Err(ModelError::Provider { message, code: Some(c), said: Some(s), .. }) if message == "the response failed" && c == "server_error" && s == "it broke"),
        "{r:?}"
    );
}
