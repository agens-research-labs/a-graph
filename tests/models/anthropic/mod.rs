use crate::common::http::{Req, Resp, serve};
use a_graph::models::*;
use a_graph::resources::{Generate, MediaKind};
use a_graph::{Path, Rule};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const KEY_VAR: &str = "A_GRAPH_TEST_ANTHROPIC_KEY";

fn api_model(var: &str) -> AnthropicModel {
    AnthropicModel {
        common: Common {
            id: "fast".into(),
            model: "claude-haiku-4-5".into(),
            input: vec![Modality::Text],
            output: vec![Modality::Text],
        },
        api_key: var.into(),
        workspace_id: None,
    }
}

fn client(base: &str) -> Anthropic {
    // SAFETY: a variable only these tests use, set to the same value every time.
    unsafe { std::env::set_var(KEY_VAR, "test-key") };
    Anthropic::new(&api_model(KEY_VAR), &Path::root().key("models").idx(0))
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
        max_tokens: Some(100),
        reasoning: None,
        tool_rounds: vec![],
    }
}

fn message(content: Value, stop: &str) -> String {
    json!({ "id": "msg_1", "type": "message", "role": "assistant", "model": "claude-haiku-4-5-20251001", "content": content,
            "stop_reason": stop, "usage": { "input_tokens": 12, "output_tokens": 7, "cache_read_input_tokens": 3, "cache_creation_input_tokens": null,
            "output_tokens_details": { "thinking_tokens": 2 } } })
    .to_string()
}

fn text_answer(text: &str) -> String {
    message(json!([{ "type": "text", "text": text }]), "end_turn")
}

/// Calls once against a server that always answers `answer`; returns the result and the request sent.
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

fn serde_norway_generate() -> Generate {
    Generate::default()
}

// --- batch ----------------------------------------------------------------------

fn batch_server(polls_before_end: usize) -> (impl Fn(&Req) -> Resp + Send + Sync + 'static, Arc<Mutex<usize>>) {
    let polls = Arc::new(Mutex::new(0usize));
    let seen = polls.clone();
    let results = [
        json!({ "custom_id": "two", "result": { "type": "succeeded", "message": serde_json::from_str::<Value>(&text_answer("second")).unwrap() } }),
        json!({ "custom_id": "one", "result": { "type": "succeeded", "message": serde_json::from_str::<Value>(&text_answer("first")).unwrap() } }),
        json!({ "custom_id": "three", "result": { "type": "errored", "error": { "type": "error", "error": { "type": "invalid_request_error", "message": "bad params" } } } }),
        json!({ "custom_id": "four", "result": { "type": "expired" } }),
    ]
    .iter()
    .map(|v| v.to_string())
    .collect::<Vec<_>>()
    .join("\n");
    let handler = move |req: &Req| match (req.method.as_str(), req.path.as_str()) {
        ("POST", "/v1/messages/batches") => {
            Resp::ok(json!({ "id": "msgbatch_1", "processing_status": "in_progress" }).to_string())
        }
        ("GET", "/v1/messages/batches/msgbatch_1") => {
            let mut n = polls.lock().unwrap();
            *n += 1;
            let state = if *n > polls_before_end { "ended" } else { "in_progress" };
            Resp::ok(json!({ "id": "msgbatch_1", "processing_status": state, "request_counts": { "succeeded": 2, "errored": 1, "expired": 1 }, "results_url": "https://elsewhere.example.com/x" }).to_string())
        }
        ("GET", "/v1/messages/batches/msgbatch_1/results") => Resp::ok(results.clone()),
        _ => Resp::status(404, "{}"),
    };
    (handler, seen)
}

// --- live (run with: cargo test -- --ignored) ------------------------------------

fn live_model() -> Anthropic {
    // an API key not scoped to a workspace needs its id: CLAUDE_WORKSPACE_ID
    let mut m = api_model("CLAUDE_API_KEY");
    m.workspace_id = std::env::var("CLAUDE_WORKSPACE_ID").ok();
    Anthropic::new(&m, &Path::root()).unwrap()
}

#[path = "plain_call/plain_call.rs"]
mod plain_call;

#[path = "workspace_id/workspace_id.rs"]
mod workspace_id;

#[path = "request_parts/request_parts.rs"]
mod request_parts;

#[path = "reasoning_levels/reasoning_levels.rs"]
mod reasoning_levels;

#[path = "tool_rounds/tool_rounds.rs"]
mod tool_rounds;

#[path = "structured/structured.rs"]
mod structured;

#[path = "stop_reasons/stop_reasons.rs"]
mod stop_reasons;

#[path = "error_messages/error_messages.rs"]
mod error_messages;

#[path = "network_error/network_error.rs"]
mod network_error;

#[path = "early_checks/early_checks.rs"]
mod early_checks;

#[path = "check_limits/check_limits.rs"]
mod check_limits;

#[path = "key_from_env/key_from_env.rs"]
mod key_from_env;

#[path = "batch_flow/batch_flow.rs"]
mod batch_flow;

#[path = "bad_request_alone/bad_request_alone.rs"]
mod bad_request_alone;

#[path = "batch_error/batch_error.rs"]
mod batch_error;

#[path = "live_call/live_call.rs"]
mod live_call;

#[path = "live_structured/live_structured.rs"]
mod live_structured;

#[path = "live_tool_round/live_tool_round.rs"]
mod live_tool_round;

#[path = "live_own_error/live_own_error.rs"]
mod live_own_error;

#[path = "batch_cancel/batch_cancel.rs"]
mod batch_cancel;

#[path = "batch_retry/batch_retry.rs"]
mod batch_retry;

#[path = "batch_let_go/batch_let_go.rs"]
mod batch_let_go;

#[path = "structured_refusal/structured_refusal.rs"]
mod structured_refusal;
