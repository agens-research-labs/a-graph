use crate::common::run_live::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Live plain answer.
#[tokio::test]
#[ignore = "needs a local Ollama with the model qwen3.5:4b"]
async fn plain_answer() {
    let out = ask(&program(here(), "plain_answer"), "Reply with the single word: pong").await;
    assert!(text(&out.assistant.message).to_lowercase().contains("pong"));
    assert!(out.info.usage.input_tokens > 0 && out.info.usage.output_tokens > 0);
    assert_eq!(out.info.provider, "ollama");
}
