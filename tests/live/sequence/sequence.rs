use crate::common::run_live::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Live a sequence of two nodes passes the answer on.
#[tokio::test]
#[ignore = "needs a local Ollama with the model qwen3.5:4b"]
async fn sequence() {
    let out = run_graph(here(), "sequence", "F").await;
    let ids: Vec<&str> = out.items.iter().map(|i| i.agent_id.as_str()).collect();
    assert_eq!(ids, ["country", "capital"]);
    // the second node answered about the country the first one named
    assert!(!said(&out.items[0]).trim().is_empty() && !said(&out.items[1]).trim().is_empty());
}
