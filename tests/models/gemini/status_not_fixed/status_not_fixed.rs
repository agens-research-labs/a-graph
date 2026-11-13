use super::*;

/// A `status` that is not a fixed code (a text, or too long) stays out of the message, as the
/// rest of what the provider wrote.
#[tokio::test]
async fn status_not_fixed() {
    let long = "a".repeat(65);
    for status in ["please wait", long.as_str()] {
        let body = json!({ "status": status, "steps": [] }).to_string();
        let (r, _) = one_call(call("q"), Resp::ok(body)).await;
        let e = r.unwrap_err();
        assert_eq!(
            e.to_string(),
            "model `flash` (gemini): invalid response: an unexpected status",
            "{e:?}"
        );
        assert_eq!(e.said(), Some(status));
    }
}
