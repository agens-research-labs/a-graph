use crate::common::run_live::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Live a markdown media is read.
#[tokio::test]
#[ignore = "needs a local Ollama with the model qwen3.5:4b"]
async fn md_media() {
    let out = ask(
        &program(here(), "md_media"),
        "What is the secret word in the note? Reply with the word only.",
    )
    .await;
    assert!(text(&out.assistant.message).to_lowercase().contains("mango"));
}
