use crate::common::clean_output;
use crate::common::run_live::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Live: what was said in the first turn reaches the model in the second, as the history.
#[tokio::test]
#[ignore = "needs a local Ollama with the model qwen3.5:4b"]
async fn live_chat_history() {
    let path = "target/test-memory/live_chat_history.json";
    clean_output(path);
    let p = program(here(), "live_chat_history");
    ask(&p, "My name is Marta. Reply with the single word: ok").await;
    let out = ask(&p, "What is my name? Answer with just the name.").await;
    assert!(text(&out.assistant.message).to_lowercase().contains("marta"));
    let record: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(
        record[0]["chat"].as_array().unwrap().len(),
        4,
        "two turns, two messages each"
    );
}
