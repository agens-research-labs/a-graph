use super::*;

/// Errors say what failed and pass mistrals message.
#[tokio::test]
async fn error_messages() {
    let (r, _) = one_call(
        call("q"),
        Resp::status(401, json!({ "message": "Unauthorized", "request_id": "x" }).to_string()),
    )
    .await;
    assert!(
        matches!(&r, Err(ModelError::Credentials { detail, said, .. }) if detail == "the provider answered 401" && said.as_deref() == Some("Unauthorized")),
        "{r:?}"
    );

    let (r, _) = one_call(
        call("q"),
        Resp::status(
            422,
            json!({ "object": "error", "message": "Invalid model: nope", "type": "invalid_model" }).to_string(),
        ),
    )
    .await;
    let e = r.unwrap_err();
    let text = e.to_string();
    assert!(
        text.contains("`mistral_small`")
            && text.contains("(mistral)")
            && text.contains("422")
            && !text.contains("Invalid model"),
        "{text}"
    );
    assert!(e.said().is_some_and(|s| s.contains("Invalid model")), "{e:?}");

    let (r, _) = one_call(
        call("q"),
        Resp::status(
            422,
            json!({ "detail": [{ "loc": ["body", "messages"], "msg": "field required" }] }).to_string(),
        ),
    )
    .await;
    let e = r.unwrap_err();
    assert!(!e.to_string().contains("field required"), "{e}");
    assert!(e.said().is_some_and(|s| s.contains("field required")), "{e:?}");

    let (r, _) = one_call(call("q"), Resp::ok("not json")).await;
    assert!(matches!(r, Err(ModelError::InvalidResponse { .. })));
    assert!(matches!(
        client("http://127.0.0.1:1").invoke(&call("q")).await,
        Err(ModelError::Network { .. })
    ));
}
