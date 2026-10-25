//! A `batch` group: the requests of its branches go to the provider as one batch job. The model is
//! an Anthropic one (it has batch) whose address is a mock server of the Anthropic batch API; the
//! graphs are yml files in the folders of the tests.

pub use std::collections::HashMap;
pub use std::sync::{Arc, Mutex};
pub use std::time::Duration;
pub use serde_json::{Value, json};
pub use a_graph::Path;
pub use a_graph::models::{Anthropic, AnthropicModel, Common, Modality};
pub use a_graph::run::{Content, Program, build};
pub use super::run::{yml_in};
pub use super::http::{Req, Resp, serve};

/// What the mock knows: the jobs it was given (id -> (custom_id, text) of each request) and the
/// request that must fail, if any.
#[derive(Default)]
pub struct Jobs {
    pub created: HashMap<String, Vec<(String, String)>>,
    pub fail: Option<String>,
}

/// The text of the question in the `params` of a request of the batch.
pub fn question(params: &Value) -> String {
    let last = params["messages"].as_array().unwrap().last().unwrap();
    last["content"].as_array().unwrap().iter().filter_map(|b| b["text"].as_str()).collect::<Vec<_>>().join("")
}

pub fn message(text: &str) -> Value {
    json!({ "id": "msg", "type": "message", "role": "assistant", "model": "claude", "content": [{ "type": "text", "text": text }], "stop_reason": "end_turn", "usage": { "input_tokens": 1, "output_tokens": 1 } })
}

/// The mock of the Anthropic API: a call answers the text it was asked; a batch is created, is
/// `ended` at the first poll, and its results answer the text of each request.
pub fn api(jobs: Arc<Mutex<Jobs>>) -> impl Fn(&Req) -> Resp + Send + Sync + 'static {
    move |req| {
        let mut jobs = jobs.lock().unwrap();
        let path = req.path.as_str();
        match (req.method.as_str(), path) {
            ("POST", "/v1/messages") => {
                let body: Value = serde_json::from_str(&req.body).unwrap();
                Resp::ok(message(&question(&body)).to_string())
            }
            ("POST", "/v1/messages/batches") => {
                let body: Value = serde_json::from_str(&req.body).unwrap();
                let entries = body["requests"].as_array().unwrap().iter().map(|r| (r["custom_id"].as_str().unwrap().to_string(), question(&r["params"]))).collect();
                let id = format!("b{}", jobs.created.len() + 1);
                jobs.created.insert(id.clone(), entries);
                Resp::ok(json!({ "id": id, "processing_status": "in_progress" }).to_string())
            }
            ("GET", p) if p.ends_with("/results") => {
                let id = p.trim_start_matches("/v1/messages/batches/").trim_end_matches("/results");
                let lines: Vec<String> = jobs.created[id]
                    .iter()
                    .map(|(custom_id, text)| {
                        if jobs.fail.as_deref() == Some(custom_id) {
                            json!({ "custom_id": custom_id, "result": { "type": "errored", "error": { "error": { "message": "this request was refused" } } } }).to_string()
                        } else {
                            json!({ "custom_id": custom_id, "result": { "type": "succeeded", "message": message(text) } }).to_string()
                        }
                    })
                    .collect();
                Resp::ok(lines.join("\n"))
            }
            ("GET", p) if p.starts_with("/v1/messages/batches/") => Resp::ok(json!({ "processing_status": "ended", "request_counts": { "succeeded": 1 } }).to_string()),
            other => Resp::status(404, format!("{other:?}")),
        }
    }
}

/// The program of a fixture, whose model is put on the mock.
pub async fn program(dir: &std::path::Path, name: &str, fail: Option<&str>) -> (Arc<Program>, Arc<Mutex<Vec<Req>>>, Arc<Mutex<Jobs>>) {
    // SAFETY: the values are never used to call anything.
    unsafe {
        std::env::set_var("ANTHROPIC_API_KEY", "a");
        std::env::set_var("A_GRAPH_TEST_RUN_BATCH_KEY", "k");
    }
    let jobs = Arc::new(Mutex::new(Jobs { fail: fail.map(str::to_string), ..Jobs::default() }));
    let (base, seen) = serve(api(jobs.clone())).await;
    let mut p = build(&yml_in(dir, name)).unwrap_or_else(|e| panic!("{e}"));
    let spec = AnthropicModel {
        common: Common { id: "m".into(), model: "claude-haiku-4-5".into(), input: vec![Modality::Text], output: vec![Modality::Text] },
        api_key: "A_GRAPH_TEST_RUN_BATCH_KEY".into(),
        workspace_id: None,
    };
    p.models.insert("m", Box::new(Anthropic::new(&spec, &Path::root()).unwrap().with_base_url(&base).with_poll_interval(Duration::from_millis(5))));
    (Arc::new(p), seen, jobs)
}

/// The requests sent to the batch API to create jobs: how many requests each one holds.
pub fn job_sizes(seen: &Arc<Mutex<Vec<Req>>>) -> Vec<usize> {
    seen.lock().unwrap().iter().filter(|r| r.method == "POST" && r.path == "/v1/messages/batches").map(|r| serde_json::from_str::<Value>(&r.body).unwrap()["requests"].as_array().unwrap().len()).collect()
}




