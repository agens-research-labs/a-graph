use super::*;

/// Live: the API accepts a correction of `schema_prompt` after a tool round.
#[tokio::test]
#[ignore = "needs a local Ollama; the model is A_GRAPH_LIVE_OLLAMA_MODEL, llama3.2:latest when unset"]
async fn live_correction() {
    let model = live(&std::env::var("A_GRAPH_LIVE_OLLAMA_MODEL").unwrap_or("llama3.2:latest".into()));
    crate::common::live_correction::correction_accepted(&model, call("")).await;
}
