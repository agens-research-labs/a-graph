use super::*;

/// Errors say what failed and pass bedrocks message.
#[tokio::test]
async fn error_messages() {
    let err = |status: u16, kind: &str, msg: &str| {
        Resp::status(status, json!({ "message": msg }).to_string()).with_error_type(kind)
    };

    let (r, _) = one_call(
        call("q"),
        err(403, "AccessDeniedException", "You don't have access to the model"),
    )
    .await;
    assert!(
        matches!(&r, Err(ModelError::Credentials { detail, said: Some(s), .. }) if detail.contains("AccessDeniedException") && !detail.contains("don't have access") && s.contains("don't have access")),
        "{r:?}"
    );

    let (r, _) = one_call(
        call("q"),
        err(400, "ValidationException", "The provided model identifier is invalid."),
    )
    .await;
    let e = r.unwrap_err();
    let text = e.to_string();
    assert!(
        text.contains("`strong`")
            && text.contains("(bedrock)")
            && text.contains("ValidationException")
            && !text.contains("model identifier is invalid"),
        "{text}"
    );
    assert!(
        e.said().is_some_and(|s| s.contains("model identifier is invalid")),
        "{e:?}"
    );

    // the SDK retries a throttling by itself: the server keeps answering it
    let (base, seen) = serve(|_| {
        Resp::status(
            429,
            json!({ "__type": "ThrottlingException", "message": "Too many requests" }).to_string(),
        )
    })
    .await;
    let r = client(&base).invoke(&call("q")).await;
    assert!(
        matches!(&r, Err(ModelError::Provider { code: Some(c), .. }) if c == "ThrottlingException"),
        "{r:?}"
    );
    assert!(seen.lock().unwrap().len() > 1, "the SDK should have retried");

    let r = client("http://127.0.0.1:1").invoke(&call("q")).await;
    assert!(
        matches!(
            r,
            Err(ModelError::Network { .. }) | Err(ModelError::Credentials { .. }) | Err(ModelError::Provider { .. })
        ),
        "{r:?}"
    );
}
