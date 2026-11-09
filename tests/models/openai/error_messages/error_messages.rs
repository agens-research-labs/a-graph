use super::*;

/// Errors say what failed and pass openais message.
#[tokio::test]
async fn error_messages() {
    let err = |status, msg: &str| {
        Resp::status(
            status,
            json!({ "error": { "message": msg, "type": "x", "param": null, "code": null } }).to_string(),
        )
    };

    let (r, _) = one_call(call("q"), err(401, "Incorrect API key provided")).await;
    assert!(
        matches!(&r, Err(ModelError::Credentials { detail, said, .. }) if detail == "the provider answered 401 (x)" && said.as_deref() == Some("Incorrect API key provided")),
        "{r:?}"
    );

    let (r, _) = one_call(
        call("q"),
        err(
            400,
            "Unsupported parameter: 'temperature' is not supported with this model.",
        ),
    )
    .await;
    let e = r.unwrap_err();
    let text = e.to_string();
    assert!(
        text.contains("`smart`")
            && text.contains("(openai)")
            && text.contains("400")
            && !text.contains("Unsupported parameter"),
        "{text}"
    );
    assert!(e.said().is_some_and(|s| s.contains("Unsupported parameter")), "{e:?}");

    let (r, _) = one_call(call("q"), err(429, "Rate limit reached")).await;
    assert!(
        matches!(&r, Err(ModelError::Provider { status: Some(429), .. })),
        "{r:?}"
    );

    let (r, _) = one_call(call("q"), Resp::ok("not json")).await;
    assert!(matches!(r, Err(ModelError::InvalidResponse { .. })));
    let r = client("http://127.0.0.1:1").invoke(&call("q")).await;
    assert!(matches!(r, Err(ModelError::Network { .. })), "{r:?}");
}
