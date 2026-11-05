use super::*;

/// With a schema, the message of a guardrail is the message.
#[tokio::test]
async fn structured_refusal() {
    let mut c = call("q");
    c.schema = Some(json!({ "type": "object" }));
    let (r, _) = one_call(
        c,
        Resp::ok(converse(
            json!([{ "text": "Sorry, I cannot answer that." }]),
            "guardrail_intervened",
        )),
    )
    .await;
    let r = r.unwrap();
    assert_eq!(
        (r.stop_reason, r.message.as_deref(), r.structured),
        (StopReason::Refused, Some("Sorry, I cannot answer that."), None)
    );
}
