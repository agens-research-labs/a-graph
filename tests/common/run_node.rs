//! Running one node. Every test is a folder with its own yml files; its model is the
//! mock server of `tests/common/http.rs` (an Ollama-style `/api/chat`), on the port the file names.
//! Nothing leaves the machine. The boards the tests change are copies in `target/test-boards/`.

pub use std::sync::{Arc, Mutex};
pub use serde_json::{Value, json};
pub use a_graph::run::{Content, NodeInput, Program, build};
pub use super::http::{Req, Resp, serve_at};
pub use super::run::{yml_in};

/// The answer of the mock: the text, and optionally a thinking text.
pub fn chat(text: &str, thinking: Option<&str>) -> String {
    let mut message = json!({ "role": "assistant", "content": text });
    if let Some(t) = thinking {
        message["thinking"] = json!(t);
    }
    json!({ "model": "m", "message": message, "done": true, "done_reason": "stop", "prompt_eval_count": 7, "eval_count": 3 }).to_string()
}

/// The port the file names for its model: `host: http://127.0.0.1:<port>`.
pub fn port_of(yml: &str) -> u16 {
    let rest = yml.split("host: http://127.0.0.1:").nth(1).expect("the fixture names no port");
    rest.chars().take_while(char::is_ascii_digit).collect::<String>().parse().unwrap()
}

/// The program of `<dir>/<name>.yml`, with the mock listening where the file says
/// and answering `answer`; and the requests the mock received.
pub async fn program(dir: &std::path::Path, name: &str, answer: Resp) -> (Program, Arc<Mutex<Vec<Req>>>) {
    let yml = yml_in(dir, name);
    let answer = Arc::new(Mutex::new(answer));
    let (_, seen) = serve_at(port_of(&yml), move |_| {
        let a = answer.lock().unwrap();
        Resp::status(a.status, a.body.clone())
    })
    .await;
    (build(&yml).unwrap_or_else(|e| panic!("{e}")), seen)
}

pub fn sent(seen: &Arc<Mutex<Vec<Req>>>) -> Value {
    serde_json::from_str(&seen.lock().unwrap()[0].body).unwrap()
}

pub fn entity(p: &Program) -> a_graph::graph::Entity {
    p.nodes.entity("n").unwrap()
}

pub fn user(text: &str) -> NodeInput {
    NodeInput { user_message: Some(text.into()), ..Default::default() }
}

pub const HEADER: &str = "---\n<description>\nWhat it is.\n<purpose>\nWhat it is for.\n---\n";














