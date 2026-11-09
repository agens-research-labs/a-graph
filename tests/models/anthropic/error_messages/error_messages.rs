use super::*;

/// Errors say what failed and pass the providers message.
#[tokio::test]
async fn error_messages() {
    let err = |status, msg: &str| {
        Resp::status(
            status,
            json!({ "type": "error", "error": { "type": "x", "message": msg } }).to_string(),
        )
    };

    let (r, _) = one_call(call("q"), err(401, "invalid x-api-key")).await;
    assert!(
        matches!(&r, Err(ModelError::Credentials { detail, said, .. }) if detail == "the provider answered 401 (x)" && said.as_deref() == Some("invalid x-api-key")),
        "{r:?}"
    );

    let (r, _) = one_call(call("q"), err(429, "slow down")).await;
    assert!(
        matches!(&r, Err(ModelError::Provider { status: Some(429), code: Some(c), said: Some(s), .. }) if c == "x" && s == "slow down"),
        "{r:?}"
    );

    // what a model does not accept is Anthropic's own message: what it said, never the message
    let msg = "\"thinking.type.disabled\" is not supported for this model.";
    let (r, _) = one_call(call("q"), err(400, msg)).await;
    let e = r.unwrap_err();
    let text = e.to_string();
    assert!(
        text.contains("`fast`") && text.contains("(anthropic)") && text.contains("400") && !text.contains(msg),
        "{text}"
    );
    assert_eq!(e.said(), Some(msg));

    let (r, _) = one_call(call("q"), Resp::status(500, "<html>oops</html>")).await;
    assert!(
        matches!(&r, Err(ModelError::Provider { message, said: Some(s), .. }) if s.contains("oops") && !message.contains("oops")),
        "{r:?}"
    );

    let (r, _) = one_call(call("q"), Resp::ok("not json")).await;
    assert!(matches!(r, Err(ModelError::InvalidResponse { .. })));
}
