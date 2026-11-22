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
    // the text comes back as it came: the JSON is read by `models::structured`, whatever the provider
    let r = r.unwrap();
    assert_eq!((r.message.as_deref(), &r.structured), (Some("{\"a\":1}"), &None));

    let (r, _) = one_call(
        c.clone(),
        Resp::ok(interaction("incomplete", json!([output("{\"a\":")]))),
    )
    .await;
    assert!(
        r.as_ref()
            .is_ok_and(|r| r.stop_reason.is_cut() && r.message.as_deref() == Some("{\"a\":")),
        "{r:?}"
    );

    let (r, _) = one_call(c, Resp::ok(text_answer("not json"))).await;
    assert!(
        r.as_ref().is_ok_and(|r| r.message.as_deref() == Some("not json")),
        "{r:?}"
    );

    // without a schema, running out of tokens is a stop reason, not an error
    let (r, _) = one_call(call("q"), Resp::ok(interaction("incomplete", json!([output("cut")])))).await;
    let r = r.unwrap();
    assert_eq!(
        (r.stop_reason, r.stop_detail.as_deref()),
        (StopReason::MaxTokens, Some("incomplete"))
    );
}
