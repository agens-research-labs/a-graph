//! Running the whole graph. Every test is a folder with its own graphs; the model is a
//! mock server (on a free port) that answers with the text it was asked, so the answer
//! of a node shows what the node received. A node `x` answers `X(<what it took>)`.

pub use super::http::{Req, Resp, listen, listen_delayed};
pub use super::run::{yml_in, yml_with};
pub use a_graph::run::{Content, Program, RunOutput, build};
pub use serde_json::{Value, json};
pub use std::sync::Arc as Shared;
pub use std::sync::atomic::{AtomicUsize, Ordering};
pub use std::sync::{Arc, Mutex};
pub use std::time::{Duration, Instant};

pub fn chat(text: &str) -> String {
    json!({ "model": "m", "message": { "role": "assistant", "content": text }, "done": true, "done_reason": "stop", "prompt_eval_count": 1, "eval_count": 1 }).to_string()
}

/// The last message of the user in a request to the mock.
pub fn user_of(req: &Req) -> String {
    let body: Value = serde_json::from_str(&req.body).unwrap();
    body["messages"]
        .as_array()
        .unwrap()
        .iter()
        .rev()
        .find(|m| m["role"] == "user")
        .unwrap()["content"]
        .as_str()
        .unwrap()
        .to_string()
}

/// The mock: answers the text it was asked; a node with an output schema gets `{ "v": <that text> }`.
pub fn echo(req: &Req) -> Resp {
    let body: Value = serde_json::from_str(&req.body).unwrap();
    let text = user_of(req);
    if body.get("format").is_some() {
        Resp::ok(chat(&json!({ "v": text }).to_string()))
    } else {
        Resp::ok(chat(&text))
    }
}

/// The program of `<dir>/<name>.yml`, with the mock model on a free port.
pub async fn program(
    dir: &std::path::Path,
    name: &str,
    handler: impl Fn(&Req) -> Resp + Send + Sync + 'static,
) -> (Shared<Program>, Arc<Mutex<Vec<Req>>>) {
    program_with(dir, name, &[], handler).await
}

/// Like `program`, with the ports of the other mock servers the test started.
pub async fn program_with(
    dir: &std::path::Path,
    name: &str,
    ports: &[(&str, u16)],
    handler: impl Fn(&Req) -> Resp + Send + Sync + 'static,
) -> (Shared<Program>, Arc<Mutex<Vec<Req>>>) {
    let (port, seen) = listen(handler).await;
    let yml = yml_with(dir, name, &[ports, &[("model", port)]].concat());
    (Shared::new(build(&yml).unwrap_or_else(|e| panic!("{e}"))), seen)
}

/// The final output as (agent id, text).
pub fn items(out: &RunOutput) -> Vec<(String, String)> {
    out.items
        .iter()
        .map(|i| match &i.message {
            Some(Content::Text(t)) => (i.agent_id.clone(), t.clone()),
            Some(Content::Json(j)) => (i.agent_id.clone(), j.to_string()),
            None => (i.agent_id.clone(), String::new()),
        })
        .collect()
}

pub fn pair(id: &str, text: &str) -> (String, String) {
    (id.to_string(), text.to_string())
}

/// What a request to the mock asked, node by node: the first letter of the text (S, I, D, E, F, Z...).
pub fn order(seen: &Arc<Mutex<Vec<Req>>>) -> String {
    seen.lock()
        .unwrap()
        .iter()
        .map(|r| user_of(r).chars().next().unwrap())
        .collect()
}

/// The text the node `F` was asked, between `F(` and the last `)`.
pub fn asked_of(seen: &Arc<Mutex<Vec<Req>>>, first: char) -> String {
    let text = seen
        .lock()
        .unwrap()
        .iter()
        .map(user_of)
        .find(|t| t.starts_with(first))
        .unwrap();
    text[2..text.len() - 1].to_string()
}

pub const D_AND_E: &str = "---\nfrom agent d\n---\nD(I(S(hi)))\n\n---\nfrom agent e\n---\nE(I(S(hi)))";

/// Like `program`, with a mock that takes 300 ms to answer the nodes `D` and `E` (the server waits
/// without blocking a thread: a handler that slept would hold up the other requests).
pub async fn program_slow(
    dir: &std::path::Path,
    name: &str,
    ports: &[(&str, u16)],
) -> (Shared<Program>, Arc<Mutex<Vec<Req>>>) {
    let delay = |req: &Req| {
        if user_of(req).starts_with(['D', 'E']) {
            Duration::from_millis(300)
        } else {
            Duration::ZERO
        }
    };
    let (port, seen) = listen_delayed(delay, |req| Resp::ok(chat(&user_of(req)))).await;
    let yml = yml_with(dir, name, &[ports, &[("model", port)]].concat());
    (Shared::new(build(&yml).unwrap_or_else(|e| panic!("{e}"))), seen)
}

// --- the branches of a `parallel` group run on different threads ----------------------------

pub use a_graph::models::{
    BatchEvents, BatchRequest, BatchReturn, Llm, ModelError, ModelInvocation, ModelRequirements, ModelResponse,
    ResponseInfo, StopReason, Usage,
};

/// A model that answers the text it is asked, after blocking its thread for 200 ms, and writes down
/// the thread it was called on.
pub struct Blocking {
    pub threads: Arc<Mutex<Vec<(char, std::thread::ThreadId)>>>,
}

#[async_trait::async_trait]
impl Llm for Blocking {
    fn check(&self, _: &ModelRequirements) -> Result<(), ModelError> {
        Ok(())
    }

    async fn invoke(&self, call: &ModelInvocation) -> Result<ModelResponse, ModelError> {
        self.threads
            .lock()
            .unwrap()
            .push((call.user.chars().next().unwrap(), std::thread::current().id()));
        std::thread::sleep(Duration::from_millis(200));
        Ok(ModelResponse {
            message: Some(call.user.clone()),
            structured: None,
            media: vec![],
            tools_call: vec![],
            reasoning: vec![],
            stop_reason: StopReason::End,
            stop_detail: None,
            info: ResponseInfo {
                provider: "test".into(),
                model: "m".into(),
                execution_time: 0.2,
                usage: Usage::default(),
            },
        })
    }

    async fn invoke_batch(&self, _: Vec<BatchRequest>, _: BatchEvents) -> BatchReturn {
        unreachable!("no batch here")
    }
}

/// The program of a fixture whose model is the `Blocking` one; no server is started.
pub fn blocking_program(
    dir: &std::path::Path,
    name: &str,
) -> (Shared<Program>, Arc<Mutex<Vec<(char, std::thread::ThreadId)>>>) {
    let mut p = build(&yml_in(dir, name)).unwrap_or_else(|e| panic!("{e}"));
    let threads = Arc::new(Mutex::new(Vec::new()));
    p.models.insert(
        "m",
        Box::new(Blocking {
            threads: threads.clone(),
        }),
    );
    (Shared::new(p), threads)
}
