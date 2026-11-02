use crate::common::run_live::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Live an image is seen.
#[tokio::test]
#[ignore = "needs a local Ollama with the model qwen3.5:4b"]
async fn image() {
    let out = ask(
        &program(here(), "image"),
        "What color is the image? Reply with one word.",
    )
    .await;
    assert!(text(&out.assistant.message).to_lowercase().contains("red"));
}
