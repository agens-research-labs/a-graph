use super::*;

/// Structured output and stop reasons.
#[tokio::test]
async fn structured_stops() {
    let mut c = call("q");
    c.schema = Some(json!({ "type": "object" }));
    let (r, req) = one_call(c.clone(), Resp::ok(text_answer("{\"a\":1}"))).await;
    assert_eq!(
        body(&req)["response_format"],
        json!({ "type": "json_schema", "json_schema": { "name": "output", "schema": { "type": "object" }, "strict": false } })
    );
    // the text comes back as it came: the JSON is read by `models::structured`, whatever the provider
    let r = r.unwrap();
    assert_eq!((r.message.as_deref(), &r.structured), (Some("{\"a\":1}"), &None));

    // with a schema, a cut answer is not an error of the provider: its text and its stop reason come back
    for finish in ["length", "model_length"] {
        let (r, _) = one_call(
            c.clone(),
            Resp::ok(completion(json!({ "role": "assistant", "content": "{\"a\":" }), finish)),
        )
        .await;
        assert!(
            r.as_ref()
                .is_ok_and(|r| r.stop_reason.is_cut() && r.message.as_deref() == Some("{\"a\":")),
            "{finish}: {r:?}"
        );
    }
    let (r, _) = one_call(c, Resp::ok(text_answer("not json"))).await;
    assert!(
        r.as_ref().is_ok_and(|r| r.message.as_deref() == Some("not json")),
        "{r:?}"
    );

    for (finish, expected) in [
        ("length", StopReason::MaxTokens),
        ("model_length", StopReason::ContextExceeded),
    ] {
        let (r, _) = one_call(
            call("q"),
            Resp::ok(completion(json!({ "role": "assistant", "content": "cut" }), finish)),
        )
        .await;
        let r = r.unwrap();
        assert_eq!((r.stop_reason, r.stop_detail.as_deref()), (expected, Some(finish)));
    }
    let (r, _) = one_call(
        call("q"),
        Resp::ok(completion(json!({ "role": "assistant", "content": "" }), "error")),
    )
    .await;
    assert!(matches!(r, Err(ModelError::Provider { .. })), "{r:?}");
}
