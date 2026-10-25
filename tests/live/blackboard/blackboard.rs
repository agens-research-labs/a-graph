use crate::common::run_live::*;
use crate::common::{fresh_copy};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Live the answer is written on the blackboard.
#[tokio::test]
#[ignore = "needs a local Ollama with the model qwen3.5:4b"]
async fn blackboard() {
    let path = "target/test-boards/run_node_live.md";
    fresh_copy(here(), "board.md", path);
    let out = ask(&program(here(), "blackboard"), "Reply with the single word: pong").await;
    let answer = text(&out.assistant.message);
    assert!(std::fs::read_to_string(path).unwrap().contains(answer.trim()), "the board holds the answer");
}
