//! A `react`: the controller answers the routing output at every turn. Every graph is a yml file in
//! the folders of the tests; the model is a mock server (on the port the file names) that plays
//! the controller by a script and answers the text it was asked to the other nodes.

pub use std::sync::atomic::{AtomicUsize, Ordering};
pub use std::sync::{Arc, Mutex};
pub use serde_json::{Value, json};
pub use a_graph::run::{Content, Program, RunOutput, build};
pub use super::run::{yml_in};
pub use super::http::{Req, Resp, serve_at};

pub fn chat(text: &str) -> String {
    json!({ "model": "m", "message": { "role": "assistant", "content": text }, "done": true, "done_reason": "stop", "prompt_eval_count": 1, "eval_count": 1 }).to_string()
}

pub fn user_of(req: &Req) -> String {
    let body: Value = serde_json::from_str(&req.body).unwrap();
    body["messages"].as_array().unwrap().iter().rev().find(|m| m["role"] == "user").unwrap()["content"].as_str().unwrap().to_string()
}

pub fn body_of(req: &Req) -> Value {
    serde_json::from_str(&req.body).unwrap()
}

/// A request of the controller carries the routing schema.
pub fn is_controller(req: &Req) -> bool {
    body_of(req)["format"]["properties"]["next_worker"].is_object()
}

/// The mock. A controller turn is answered by `turn(n, last_user_text)`, where `n` counts the turns;
/// every other node answers the text it was asked.
pub fn script(turn: impl Fn(usize, &str) -> Value + Send + Sync + 'static) -> impl Fn(&Req) -> Resp + Send + Sync + 'static {
    let turns = AtomicUsize::new(0);
    move |req| {
        if is_controller(req) {
            let n = turns.fetch_add(1, Ordering::SeqCst);
            Resp::ok(chat(&turn(n, &user_of(req)).to_string()))
        } else {
            Resp::ok(chat(&user_of(req)))
        }
    }
}

pub fn port_of(yml: &str) -> u16 {
    let rest = yml.split("host: http://127.0.0.1:").nth(1).expect("the fixture names no port");
    rest.chars().take_while(char::is_ascii_digit).collect::<String>().parse().unwrap()
}

pub async fn program(dir: &std::path::Path, name: &str, handler: impl Fn(&Req) -> Resp + Send + Sync + 'static) -> (Arc<Program>, Arc<Mutex<Vec<Req>>>) {
    let yml = yml_in(dir, name);
    let (_, seen) = serve_at(port_of(&yml), handler).await;
    (Arc::new(build(&yml).unwrap_or_else(|e| panic!("{e}"))), seen)
}

pub fn ids(out: &RunOutput) -> Vec<&str> {
    out.items.iter().map(|i| i.agent_id.as_str()).collect()
}

pub fn text(out: &RunOutput, id: &str) -> String {
    match &out.items.iter().find(|i| i.agent_id == id).unwrap().message {
        Some(Content::Text(t)) => t.clone(),
        other => panic!("{other:?}"),
    }
}

/// The requests to the controller, in order.
pub fn controller_turns(seen: &Arc<Mutex<Vec<Req>>>) -> Vec<Value> {
    seen.lock().unwrap().iter().filter(|r| is_controller(r)).map(body_of).collect()
}







