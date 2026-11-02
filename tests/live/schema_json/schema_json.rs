use crate::common::run_live::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Live a schema gives json.
#[tokio::test]
#[ignore = "needs a local Ollama with the model qwen3.5:4b"]
async fn schema_json() {
    let out = ask(
        &program(here(), "schema_json"),
        "What is the capital of France? Put it in `answer`.",
    )
    .await;
    let Some(Content::Json(json)) = out.assistant.message else {
        panic!("expected JSON")
    };
    assert!(
        json["answer"].as_str().unwrap().to_lowercase().contains("paris"),
        "{json}"
    );
}
