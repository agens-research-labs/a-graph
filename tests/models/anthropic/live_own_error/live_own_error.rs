use super::*;

/// Live what a model does not accept comes back as its own message.
#[tokio::test]
#[ignore = "calls the real API with CLAUDE_API_KEY"]
async fn live_own_error() {
    // haiku 4.5 only knows the old thinking mode: adaptive is refused by Anthropic, not by us
    let mut c = call("hi");
    c.reasoning = Some(Reasoning::Low);
    let err = live_model().invoke(&c).await.unwrap_err().to_string();
    assert!(err.contains("`fast`") && err.contains("(anthropic)") && err.contains("400"), "{err}");
}
