use super::*;

/// Structured output.
#[tokio::test]
async fn structured() {
    let mut c = call("q");
    c.schema = Some(json!({ "type": "object" }));

    let (r, _) = one_call(c.clone(), Resp::ok(text_answer("{\"a\":1}"))).await;
    // the text comes back as it came: the JSON is read by `models::structured`, whatever the provider
    let r = r.unwrap();
    assert_eq!((r.message.as_deref(), &r.structured), (Some("{\"a\":1}"), &None));

    // with a schema, a cut answer is not an error of the provider: its text and its stop reason come back
    for stop in ["max_tokens", "model_context_window_exceeded"] {
        let (r, _) = one_call(
            c.clone(),
            Resp::ok(message(json!([{ "type": "text", "text": "{\"a\":" }]), stop)),
        )
        .await;
        assert!(
            r.as_ref()
                .is_ok_and(|r| r.stop_reason.is_cut() && r.message.as_deref() == Some("{\"a\":")),
            "{stop}: {r:?}"
        );
    }

    let (r, _) = one_call(c.clone(), Resp::ok(text_answer("not json"))).await;
    assert!(
        r.as_ref().is_ok_and(|r| r.message.as_deref() == Some("not json")),
        "{r:?}"
    );

    // without a schema, max_tokens is a stop reason, not an error
    let (r, _) = one_call(
        call("q"),
        Resp::ok(message(json!([{ "type": "text", "text": "cut" }]), "max_tokens")),
    )
    .await;
    assert_eq!(r.unwrap().stop_reason, StopReason::MaxTokens);
}
