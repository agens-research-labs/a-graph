use crate::common::http::{Req, Resp, serve};
use a_graph::models::*;
use a_graph::resources::{Generate, MediaKind, MediaProvider, OpenaiMedia, Size};
use a_graph::{Path, Rule};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const KEY_VAR: &str = "A_GRAPH_TEST_MISTRAL_KEY";

fn api_model(var: &str) -> ApiModel {
    ApiModel {
        common: Common {
            id: "mistral_small".into(),
            model: "mistral-small-latest".into(),
            input: vec![Modality::Text],
            output: vec![Modality::Text],
        },
        api_key: var.into(),
    }
}

fn client(base: &str) -> Mistral {
    // SAFETY: a variable only these tests use, set to the same value every time.
    unsafe { std::env::set_var(KEY_VAR, "test-key") };
    Mistral::new(&api_model(KEY_VAR), &Path::root().key("models").idx(0))
        .unwrap()
        .with_base_url(base)
        .with_poll_interval(Duration::from_millis(10))
}

fn call(user: &str) -> ModelInvocation {
    ModelInvocation {
        system: String::new(),
        history: vec![],
        user: user.into(),
        media: vec![],
        tools: vec![],
        output_media: vec![],
        schema: None,
        temperature: None,
        top_p: None,
        max_tokens: None,
        reasoning: None,
        tool_rounds: vec![],
    }
}

fn generate() -> Generate {
    Generate {
        voice: None,
        language_code: None,
        size: None,
        quality: None,
        background: None,
    }
}

fn completion(message: Value, finish: &str) -> String {
    json!({ "id": "c1", "object": "chat.completion", "model": "mistral-small-2506", "created": 1,
            "choices": [{ "index": 0, "message": message, "finish_reason": finish }],
            "usage": { "prompt_tokens": 30, "completion_tokens": 9, "total_tokens": 39, "completion_tokens_details": { "reasoning_tokens": 4 } } })
    .to_string()
}

fn text_answer(text: &str) -> String {
    completion(json!({ "role": "assistant", "content": text }), "stop")
}

async fn one_call(call: ModelInvocation, answer: Resp) -> (Result<ModelResponse, ModelError>, Req) {
    let answer = Arc::new(Mutex::new(Some(answer)));
    let (base, seen) = serve(move |_| answer.lock().unwrap().take().unwrap_or(Resp::ok("{}"))).await;
    let result = client(&base).invoke(&call).await;
    let req = seen.lock().unwrap()[0].clone();
    (result, req)
}

fn body(req: &Req) -> Value {
    serde_json::from_str(&req.body).unwrap()
}

// --- batch ---------------------------------------------------------------------------------

fn batch_server(
    polls_before_done: usize,
    final_status: &'static str,
) -> (impl Fn(&Req) -> Resp + Send + Sync + 'static, Arc<Mutex<usize>>) {
    let polls = Arc::new(Mutex::new(0usize));
    let seen = polls.clone();
    let ok_line = |id: &str, text: &str| {
        json!({ "id": "x", "custom_id": id, "response": { "status_code": 200, "body": serde_json::from_str::<Value>(&text_answer(text)).unwrap() }, "error": null }).to_string()
    };
    let out = [
        ok_line("two", "second"),
        ok_line("one", "first"),
        json!({ "custom_id": "three", "response": { "status_code": 422, "body": { "message": "bad params" } }, "error": null }).to_string(),
        json!({ "custom_id": "four", "response": null, "error": { "message": "timed out" } }).to_string(),
    ]
    .join("\n");
    let handler = move |req: &Req| match (req.method.as_str(), req.path.as_str()) {
        ("POST", "/v1/batch/jobs") => Resp::ok(json!({ "id": "job_1", "status": "QUEUED" }).to_string()),
        ("GET", "/v1/batch/jobs/job_1") => {
            let mut n = polls.lock().unwrap();
            *n += 1;
            let status = if *n > polls_before_done {
                final_status
            } else {
                "RUNNING"
            };
            let file = if final_status == "SUCCESS" {
                json!("file-out")
            } else {
                Value::Null
            };
            Resp::ok(json!({ "id": "job_1", "status": status, "output_file": file, "error_file": null, "errors": [{ "message": "the job was invalid", "count": 1 }] }).to_string())
        }
        ("GET", "/v1/files/file-out/content") => Resp::ok(out.clone()),
        _ => Resp::status(404, "{}"),
    };
    (handler, seen)
}

#[path = "plain_call/plain_call.rs"]
mod plain_call;

#[path = "reasoning_levels/reasoning_levels.rs"]
mod reasoning_levels;

#[path = "thinking_tools/thinking_tools.rs"]
mod thinking_tools;

#[path = "structured_stops/structured_stops.rs"]
mod structured_stops;

#[path = "error_messages/error_messages.rs"]
mod error_messages;

#[path = "speech_out/speech_out.rs"]
mod speech_out;

#[path = "check_limits/check_limits.rs"]
mod check_limits;

#[path = "key_from_env/key_from_env.rs"]
mod key_from_env;

#[path = "batch_inline/batch_inline.rs"]
mod batch_inline;

#[path = "failed_batch/failed_batch.rs"]
mod failed_batch;

#[path = "batch_cancel/batch_cancel.rs"]
mod batch_cancel;

#[path = "batch_retry/batch_retry.rs"]
mod batch_retry;

#[path = "batch_let_go/batch_let_go.rs"]
mod batch_let_go;

#[path = "cut_tool_call/cut_tool_call.rs"]
mod cut_tool_call;

#[path = "validation_input/validation_input.rs"]
mod validation_input;
