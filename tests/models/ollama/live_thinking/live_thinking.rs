use super::*;

/// Live thinking.
#[tokio::test]
#[ignore = "needs a local Ollama with gpt-oss"]
async fn live_thinking() {
    // gpt-oss takes the levels low, medium and high, and always thinks
    let model = live("gpt-oss:latest");
    let mut c = call("What is 7*8? Answer with just the number.");
    c.max_tokens = Some(2000);
    c.reasoning = Some(Reasoning::Low);
    let r = model.invoke(&c).await.unwrap();
    assert!(r.message.unwrap().contains("56"));
    assert!(!r.reasoning.is_empty(), "no thinking came back");
}
