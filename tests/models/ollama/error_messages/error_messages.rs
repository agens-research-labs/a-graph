use super::*;

/// Errors say what failed and pass the servers message.
#[tokio::test]
async fn error_messages() {
    let (r, _) = one_call(
        call("q"),
        Resp::status(404, json!({ "error": "model 'nope:1b' not found" }).to_string()),
    )
    .await;
    let e = r.unwrap_err();
    let text = e.to_string();
    assert!(
        text.contains("`local`") && text.contains("(ollama)") && text.contains("404") && !text.contains("nope:1b"),
        "{text}"
    );
    assert_eq!(e.said(), Some("model 'nope:1b' not found"));

    let (r, _) = one_call(call("q"), Resp::ok("not json")).await;
    assert!(matches!(r, Err(ModelError::InvalidResponse { .. })));

    let r = Ollama::ollama(&local("http://127.0.0.1:1"), &Path::root())
        .unwrap()
        .invoke(&call("q"))
        .await;
    assert!(matches!(r, Err(ModelError::Network { .. })), "{r:?}");
}
