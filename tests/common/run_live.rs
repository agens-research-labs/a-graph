//! Running a node against a real, local Ollama with the model `qwen3.5:4b`
//! (`ollama pull qwen3.5:4b`). They are `#[ignore]`d because they need Ollama running on this
//! computer; run them with `cargo test --test run live_ -- --ignored`. Every program is a yml
//! file in the folder of the test. The answers of a model change from one call to
//! another, so the tests check the shape of the answer and a few words that cannot be missed.

pub use a_graph::run::{Content, NodeInput, Program, build};
pub use super::run::{yml_in, yml_with};

pub fn program(dir: &std::path::Path, name: &str) -> Program {
    build(&yml_in(dir, name)).unwrap_or_else(|e| panic!("{e}"))
}

pub async fn ask(p: &Program, question: &str) -> a_graph::run::NodeOutput {
    let node = p.nodes.entity("n").unwrap();
    p.run_node(node, NodeInput { user_message: Some(question.into()), ..Default::default() }).await.unwrap_or_else(|e| panic!("{e}"))
}

pub fn text(c: &Option<Content>) -> String {
    match c {
        Some(Content::Text(t)) => t.clone(),
        other => panic!("expected a text, got {other:?}"),
    }
}

// --- the whole graph -----------------------------------------------------------------------

pub async fn run_graph(dir: &std::path::Path, name: &str, message: &str) -> a_graph::run::RunOutput {
    std::sync::Arc::new(build(&yml_in(dir, name)).unwrap_or_else(|e| panic!("{e}"))).run(message).await.unwrap_or_else(|e| panic!("{e}"))
}

pub fn said(item: &a_graph::run::OutputItem) -> String {
    match &item.message {
        Some(Content::Text(t)) => t.to_lowercase(),
        other => panic!("expected a text, got {other:?}"),
    }
}









