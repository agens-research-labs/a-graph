use super::*;

/// Structured output and stop reasons.
#[tokio::test]
async fn structured_stops() {
    let mut c = call("q");
    c.schema = Some(json!({ "type": "object" }));
    let (r, req) = one_call(c.clone(), Resp::ok(text_answer("{\"a\":1}"))).await;
    // Converse takes the schema as a JSON string
    assert_eq!(
        body(&req)["outputConfig"]["textFormat"],
        json!({ "type": "json_schema", "structure": { "jsonSchema": { "schema": "{\"type\":\"object\"}", "name": "output" } } })
    );
    // the text comes back as it came: the JSON is read by `models::structured`, whatever the provider
    let r = r.unwrap();
    assert_eq!((r.message.as_deref(), &r.structured), (Some("{\"a\":1}"), &None));

    // with a schema, a cut answer is not an error of the provider: its text and its stop reason come back
    for stop in ["max_tokens", "model_context_window_exceeded"] {
        let (r, _) = one_call(c.clone(), Resp::ok(converse(json!([{ "text": "{\"a\":" }]), stop))).await;
        assert!(
            r.as_ref()
                .is_ok_and(|r| r.stop_reason.is_cut() && r.message.as_deref() == Some("{\"a\":")),
            "{stop}: {r:?}"
        );
    }
    let (r, _) = one_call(c, Resp::ok(text_answer("not json"))).await;
    assert!(
        r.as_ref().is_ok_and(|r| r.message.as_deref() == Some("not json")),
        "{r:?}"
    );

    for (stop, expected) in [
        ("max_tokens", StopReason::MaxTokens),
        ("model_context_window_exceeded", StopReason::ContextExceeded),
        ("guardrail_intervened", StopReason::Refused),
        ("content_filtered", StopReason::Refused),
        ("malformed_tool_use", StopReason::Other),
        ("stop_sequence", StopReason::End),
    ] {
        let (r, _) = one_call(call("q"), Resp::ok(converse(json!([{ "text": "x" }]), stop))).await;
        let r = r.unwrap();
        assert_eq!((r.stop_reason, r.stop_detail.as_deref()), (expected, Some(stop)));
    }
}
