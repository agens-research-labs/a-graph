use super::*;

/// Errors say what failed and pass the providers message.
#[tokio::test]
async fn error_messages() {
    let err = |status, msg: &str| Resp::status(status, json!({ "type": "error", "error": { "type": "x", "message": msg } }).to_string());

    let (r, _) = one_call(call("q"), err(401, "invalid x-api-key")).await;
    assert!(matches!(&r, Err(ModelError::Credentials { detail, .. }) if detail == "invalid x-api-key"), "{r:?}");

    let (r, _) = one_call(call("q"), err(429, "slow down")).await;
    assert!(matches!(&r, Err(ModelError::Provider { status: Some(429), message, .. }) if message == "slow down"), "{r:?}");

    // what a model does not accept is Anthropic's own message, shown as it is
    let msg = "\"thinking.type.disabled\" is not supported for this model.";
    let (r, _) = one_call(call("q"), err(400, msg)).await;
    let text = r.unwrap_err().to_string();
    assert!(text.contains("`fast`") && text.contains("(anthropic)") && text.contains("400") && text.contains(msg), "{text}");

    let (r, _) = one_call(call("q"), Resp::status(500, "<html>oops</html>")).await;
    assert!(matches!(&r, Err(ModelError::Provider { message, .. }) if message.contains("oops")), "{r:?}");

    let (r, _) = one_call(call("q"), Resp::ok("not json")).await;
    assert!(matches!(r, Err(ModelError::InvalidResponse { .. })));
}
