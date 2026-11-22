//! Running one node. Every test is a folder with its own yml files; its model is the
//! mock server of `tests/common/http.rs` (an Ollama-style `/api/chat`), on a free port.
//! Nothing leaves the machine. The boards the tests change are copies in `target/test-boards/`.

pub use super::http::{Req, Resp, listen};
pub use super::run::{yml_in, yml_with};
pub use a_graph::run::{Content, NodeInput, Program, build};
pub use serde_json::{Value, json};
pub use std::sync::{Arc, Mutex};

/// The answer of the mock: the text, and optionally a thinking text.
pub fn chat(text: &str, thinking: Option<&str>) -> String {
    let mut message = json!({ "role": "assistant", "content": text });
    if let Some(t) = thinking {
        message["thinking"] = json!(t);
    }
    json!({ "model": "m", "message": message, "done": true, "done_reason": "stop", "prompt_eval_count": 7, "eval_count": 3 }).to_string()
}

/// The program of `<dir>/<name>.yml`, with the mock model on a free port answering `answer`;
/// and the requests the mock received.
pub async fn program(dir: &std::path::Path, name: &str, answer: Resp) -> (Program, Arc<Mutex<Vec<Req>>>) {
    program_with(dir, name, &[], answer).await
}

/// Like `program`, with the ports of the other mock servers the test started (embedding, vectordb...).
pub async fn program_with(
    dir: &std::path::Path,
    name: &str,
    ports: &[(&str, u16)],
    answer: Resp,
) -> (Program, Arc<Mutex<Vec<Req>>>) {
    let answer = Arc::new(Mutex::new(answer));
    let (port, seen) = listen(move |_| {
        let a = answer.lock().unwrap();
        Resp::status(a.status, a.body.clone())
    })
    .await;
    let yml = yml_with(dir, name, &[ports, &[("model", port)]].concat());
    (build(&yml).unwrap_or_else(|e| panic!("{e}")), seen)
}

/// Like `program`, with a mock that answers the n-th request with the n-th of `answers` (the last one from
/// then on): enough to follow a node that calls its model more than once.
pub async fn program_seq(dir: &std::path::Path, name: &str, answers: Vec<String>) -> (Program, Arc<Mutex<Vec<Req>>>) {
    let next = std::sync::atomic::AtomicUsize::new(0);
    let (port, seen) = listen(move |_| {
        let n = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Resp::ok(answers[n.min(answers.len() - 1)].clone())
    })
    .await;
    let yml = yml_with(dir, name, &[("model", port)]);
    (build(&yml).unwrap_or_else(|e| panic!("{e}")), seen)
}

/// The body of the n-th request the mock received.
pub fn sent_at(seen: &Arc<Mutex<Vec<Req>>>, n: usize) -> Value {
    serde_json::from_str(&seen.lock().unwrap()[n].body).unwrap()
}

pub fn sent(seen: &Arc<Mutex<Vec<Req>>>) -> Value {
    serde_json::from_str(&seen.lock().unwrap()[0].body).unwrap()
}

pub fn entity(p: &Program) -> a_graph::graph::Entity {
    p.nodes.entity("n").unwrap()
}

pub fn user(text: &str) -> NodeInput {
    NodeInput {
        user_message: Some(text.into()),
        ..Default::default()
    }
}

pub const HEADER: &str = "---\n<description>\nWhat it is.\n<purpose>\nWhat it is for.\n---\n";
