use crate::common::run_live::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Live the reasoning goes to the final output.
#[tokio::test]
#[ignore = "needs a local Ollama with the model qwen3.5:4b"]
async fn reasoning() {
    let out = ask(&program(here(), "reasoning"), "What is 17 plus 25? Answer with the number.").await;
    assert!(text(&out.assistant.message).contains("42"));
    assert!(!out.assistant.reasoning.is_empty(), "the model reasoned and the node asked to show it");
    assert!(out.node.reasoning.is_empty());
}
