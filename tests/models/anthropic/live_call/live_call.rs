use super::*;

/// Live call.
#[tokio::test]
#[ignore = "calls the real API with CLAUDE_API_KEY"]
async fn live_call() {
    let mut c = call("Reply with exactly: ok");
    c.max_tokens = Some(20);
    let r = live_model().invoke(&c).await.unwrap();
    assert!(r.message.unwrap().to_lowercase().contains("ok"));
    assert!(r.info.usage.input_tokens > 0 && r.info.usage.output_tokens > 0);
}
