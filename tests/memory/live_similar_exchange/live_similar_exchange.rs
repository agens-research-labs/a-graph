use crate::common::clean_output;
use crate::common::run_live::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Live: of three saved exchanges the node gets the one nearest to the question, and the answer uses it.
#[tokio::test]
#[ignore = "needs a local Ollama with the models qwen3.5:4b and embeddinggemma"]
async fn live_similar_exchange() {
    let path = "target/test-memory/live_similar_exchange.json";
    clean_output(path);
    let p = program(here(), "live_similar_exchange");
    let saved = [
        ("My cat is called Milo.", "What a lovely name for a cat."),
        ("The tax declaration is due in November.", "Noted."),
        ("I like green tea in the morning.", "A good choice."),
    ];
    let questions: Vec<String> = saved.iter().map(|(q, _)| q.to_string()).collect();
    let vectors = p.embeddings.embed("emb", &questions).await.unwrap();
    let chat: Vec<serde_json::Value> = saved
        .iter()
        .zip(&vectors)
        .flat_map(|((q, a), v)| {
            [
                serde_json::json!({ "role": "user", "content": q, "embedding": v }),
                serde_json::json!({ "role": "assistant", "content": a }),
            ]
        })
        .collect();
    std::fs::write(
        path,
        serde_json::json!([{ "id": "n", "timestamp": "t", "synt": "", "chat": chat }]).to_string(),
    )
    .unwrap();

    let out = ask(&p, "What is the name of my cat? Answer with just the name.").await;
    assert!(text(&out.assistant.message).to_lowercase().contains("milo"));
    let record: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let last_question = &record[0]["chat"].as_array().unwrap()[6];
    assert_eq!(
        last_question["embedding"].as_array().unwrap().len(),
        768,
        "the new question was saved with its vector"
    );
}
