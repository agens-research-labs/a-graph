use super::*;

/// Live what a model does not accept comes back as the servers own message.
#[tokio::test]
#[ignore = "needs a local Ollama with llama3.2"]
async fn live_own_error() {
    let mut c = call("hi");
    c.reasoning = Some(Reasoning::Low);
    let err = live("llama3.2:latest").invoke(&c).await.unwrap_err().to_string();
    assert!(
        err.contains("does not support thinking") && err.contains("(ollama)"),
        "{err}"
    );

    let err = live("nope:1b").invoke(&call("hi")).await.unwrap_err().to_string();
    assert!(err.contains("not found") && err.contains("404"), "{err}");
}
