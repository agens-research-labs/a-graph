use super::*;

/// A JSON cut at `max_tokens` and one cut at the end of the context window are told apart.
#[tokio::test]
async fn cut_reason() {
    let mut c = call("q");
    c.schema = Some(json!({ "type": "object" }));
    for (stop, cut) in [
        ("max_tokens", Cut::MaxTokens),
        ("model_context_window_exceeded", Cut::ContextWindow),
    ] {
        let (r, _) = one_call(
            c.clone(),
            Resp::ok(message(json!([{ "type": "text", "text": "{\"a\":" }]), stop)),
        )
        .await;
        assert!(
            matches!(r, Err(ModelError::Truncated { cut: got, .. }) if got == cut),
            "{stop}: {r:?}"
        );
    }
}
