use super::*;

/// With a schema, the text of a refusal is the message.
#[tokio::test]
async fn structured_refusal() {
    let mut c = call("q");
    c.schema = Some(json!({ "type": "object" }));
    let (r, _) = one_call(
        c,
        Resp::ok(message(
            json!([{ "type": "text", "text": "I cannot help with that." }]),
            "refusal",
        )),
    )
    .await;
    let r = r.unwrap();
    assert_eq!(
        (r.stop_reason, r.message.as_deref(), r.structured),
        (StopReason::Refused, Some("I cannot help with that."), None)
    );
}
