use crate::common::run_live::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Live a parallel group packs the answers for its out node.
#[tokio::test]
#[ignore = "needs a local Ollama with the model qwen3.5:4b"]
async fn parallel_group() {
    let out = run_graph(here(), "parallel_group", "J").await;
    let ids: Vec<&str> = out.items.iter().map(|i| i.agent_id.as_str()).collect();
    assert_eq!(ids, ["start", "input", "capital", "continent", "summary", "end"]);
    assert!(out.items.iter().all(|i| !said(i).trim().is_empty()));
}
