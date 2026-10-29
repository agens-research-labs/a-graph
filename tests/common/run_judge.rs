//! A validation node. Every test is a folder with its own graph; the model is a mock
//! server (on a free port) that answers the text it was asked to the ordinary nodes and
//! a verdict to the judge, which is the request that carries the schema of the verdict.

pub use std::sync::{Arc, Mutex};
pub use serde_json::{Value, json};
pub use a_graph::run::{Content, NodeInput, Program, RunOutput, build};
pub use super::run::{yml_in, yml_with};
pub use super::http::{Req, Resp, listen};

pub fn chat(text: &str) -> String {
    json!({ "model": "m", "message": { "role": "assistant", "content": text }, "done": true, "done_reason": "stop", "prompt_eval_count": 1, "eval_count": 1 }).to_string()
}

pub fn body_of(req: &Req) -> Value {
    serde_json::from_str(&req.body).unwrap()
}

pub fn user_of(req: &Req) -> String {
    body_of(req)["messages"].as_array().unwrap().iter().rev().find(|m| m["role"] == "user").unwrap()["content"].as_str().unwrap().to_string()
}

/// The mock: the judge answers `verdict`; every other node answers the text it was asked.
pub fn mock(verdict: Value) -> impl Fn(&Req) -> Resp + Send + Sync + 'static {
    move |req| {
        let judging = body_of(req)["format"]["properties"]["validation"].is_object();
        Resp::ok(chat(&if judging { verdict.to_string() } else { user_of(req) }))
    }
}

pub fn approved() -> Value {
    json!({ "validation": true, "reasons": "", "log": "" })
}

pub fn refused() -> Value {
    json!({ "validation": false, "reasons": "too long", "log": "checked the length" })
}

/// The program of `<dir>/<name>.yml`, with the mock model on a free port.
pub async fn program(dir: &std::path::Path, name: &str, verdict: Value) -> (Arc<Program>, Arc<Mutex<Vec<Req>>>) {
    let (port, seen) = listen(mock(verdict)).await;
    let yml = yml_with(dir, name, &[("model", port)]);
    (Arc::new(build(&yml).unwrap_or_else(|e| panic!("{e}"))), seen)
}

pub fn items(out: &RunOutput) -> Vec<(String, String)> {
    out.items.iter().map(|i| (i.agent_id.clone(), match &i.message { Some(Content::Text(t)) => t.clone(), _ => String::new() })).collect()
}

pub fn pair(id: &str, text: &str) -> (String, String) {
    (id.to_string(), text.to_string())
}

/// What the mock was asked, node by node: the first letter of each text.
pub fn order(seen: &Arc<Mutex<Vec<Req>>>) -> String {
    seen.lock().unwrap().iter().map(|r| user_of(r).chars().next().unwrap()).collect()
}

pub fn asked(seen: &Arc<Mutex<Vec<Req>>>, first: char) -> String {
    seen.lock().unwrap().iter().map(user_of).find(|t| t.starts_with(first)).unwrap()
}







