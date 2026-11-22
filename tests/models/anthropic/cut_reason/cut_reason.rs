use super::*;

/// A tool call cut at `max_tokens` and one cut at the end of the context window are told apart.
#[tokio::test]
async fn cut_reason() {
    for (stop, cut) in [
        ("max_tokens", Cut::MaxTokens),
        ("model_context_window_exceeded", Cut::ContextWindow),
    ] {
        let answer = message(
            json!([{ "type": "tool_use", "id": "toolu_1", "name": "get_weather", "input": { "city": "Ro" } }]),
            stop,
        );
        let (r, _) = one_call(call("weather in Rome?"), Resp::ok(answer)).await;
        assert!(
            matches!(r, Err(ModelError::Truncated { cut: got, .. }) if got == cut),
            "{stop}: {r:?}"
        );
    }
}
