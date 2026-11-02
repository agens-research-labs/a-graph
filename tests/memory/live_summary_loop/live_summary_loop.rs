use crate::common::clean_output;
use crate::common::run_live::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Live: after two turns the summary is written and `chat` is empty; the third turn answers from the summary alone.
#[tokio::test]
#[ignore = "needs a local Ollama with the model qwen3.5:4b"]
async fn live_summary_loop() {
    let path = "target/test-memory/live_summary_loop.json";
    clean_output(path);
    let p = program(here(), "live_summary_loop");
    ask(
        &p,
        "My name is Marta and I live in Turin. Reply with the single word: ok",
    )
    .await;
    ask(&p, "I have a cat called Milo. Reply with the single word: ok").await;
    let record: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert!(
        !record[0]["synt"].as_str().unwrap().trim().is_empty(),
        "the summary was written"
    );
    assert_eq!(
        record[0]["chat"].as_array().unwrap().len(),
        0,
        "the exchanges left chat"
    );
    let out = ask(&p, "What is my cat called? Answer with just the name.").await;
    assert!(text(&out.assistant.message).to_lowercase().contains("milo"));
}
