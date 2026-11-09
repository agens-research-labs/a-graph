use super::*;

/// A code of the provider that is not a fixed code (a text, or too long) stays out of the message
/// as the rest of what the provider wrote.
#[tokio::test]
async fn code_not_fixed() {
    let long = "a".repeat(65);
    for code in ["the patient has diabetes", long.as_str()] {
        let body = json!({ "type": "error", "error": { "type": code, "message": "boom" } }).to_string();
        let (r, _) = one_call(call("q"), Resp::status(400, body)).await;
        let e = r.unwrap_err();
        assert_eq!(
            e.to_string(),
            "model `fast` (anthropic): the provider answered 400",
            "{e:?}"
        );
        assert_eq!(e.said(), Some("boom"));
    }
    let body = json!({ "type": "error", "error": { "type": "invalid_request_error", "message": "boom" } }).to_string();
    let (r, _) = one_call(call("q"), Resp::status(400, body)).await;
    assert_eq!(
        r.unwrap_err().to_string(),
        "model `fast` (anthropic): the provider answered 400 (invalid_request_error)"
    );
}
