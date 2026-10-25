use super::*;

/// Errors say what failed and pass googles message.
#[tokio::test]
async fn error_messages() {
    let (r, _) = one_call(call("q"), Resp::status(404, json!({ "error": { "message": "models/x is no longer available", "code": "not_found" } }).to_string())).await;
    let text = r.unwrap_err().to_string();
    assert!(text.contains("`flash`") && text.contains("(gemini)") && text.contains("404") && text.contains("no longer available"), "{text}");

    // a bad key comes back as 400, inside an array
    let bad_key = json!([{ "error": { "code": 400, "message": "API key not valid.", "status": "INVALID_ARGUMENT", "details": [{ "reason": "API_KEY_INVALID" }] } }]).to_string();
    let (r, _) = one_call(call("q"), Resp::status(400, bad_key)).await;
    assert!(matches!(&r, Err(ModelError::Credentials { detail, .. }) if detail == "API key not valid."), "{r:?}");

    let (r, _) = one_call(call("q"), Resp::status(503, json!({ "error": { "code": 503, "message": "high demand", "status": "UNAVAILABLE" } }).to_string())).await;
    assert!(matches!(&r, Err(ModelError::Provider { status: Some(503), message, .. }) if message == "high demand"), "{r:?}");

    let failed = json!({ "status": "failed", "steps": [], "errors": [{ "code": "x", "message": "the model failed" }] }).to_string();
    let (r, _) = one_call(call("q"), Resp::ok(failed)).await;
    assert!(matches!(&r, Err(ModelError::Provider { message, .. }) if message == "the model failed"), "{r:?}");

    let (r, _) = one_call(call("q"), Resp::ok("not json")).await;
    assert!(matches!(r, Err(ModelError::InvalidResponse { .. })));

    let r = client("http://127.0.0.1:1").invoke(&call("q")).await;
    assert!(matches!(r, Err(ModelError::Network { .. })), "{r:?}");
}
