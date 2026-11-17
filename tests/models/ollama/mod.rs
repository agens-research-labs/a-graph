use crate::common::http::{Req, Resp, serve};
use a_graph::Path;
use a_graph::models::*;
use a_graph::resources::{Generate, MediaKind};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

fn local(host: &str) -> LocalModel {
    LocalModel {
        common: Common {
            id: "local".into(),
            model: "llama3.2:latest".into(),
            input: vec![Modality::Text],
            output: vec![Modality::Text],
        },
        host: host.into(),
    }
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
        logprobs: None,
        tool_rounds: vec![],
    }
}

fn chat_answer(message: Value, done_reason: &str) -> String {
    json!({ "model": "llama3.2:latest", "message": message, "done": true, "done_reason": done_reason, "prompt_eval_count": 33, "prompt_eval_cached_count": 10, "eval_count": 7 }).to_string()
}

fn text_answer(text: &str) -> String {
    chat_answer(json!({ "role": "assistant", "content": text }), "stop")
}

async fn one_call(call: ModelInvocation, answer: Resp) -> (Result<ModelResponse, ModelError>, Req) {
    let answer = Arc::new(Mutex::new(Some(answer)));
    let (base, seen) = serve(move |_| answer.lock().unwrap().take().unwrap_or(Resp::ok("{}"))).await;
    let result = Ollama::ollama(&local(&base), &Path::root())
        .unwrap()
        .invoke(&call)
        .await;
    let req = seen.lock().unwrap()[0].clone();
    (result, req)
}

fn body(req: &Req) -> Value {
    serde_json::from_str(&req.body).unwrap()
}

// --- live: the local Ollama (run with: cargo test -- --ignored) ------------------

fn live(model: &str) -> Ollama {
    let mut m = local("http://localhost:11434");
    m.common.model = model.into();
    Ollama::ollama(&m, &Path::root()).unwrap()
}

/// A valid 1x1 red PNG.
const RED_PNG: &str =
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";

#[path = "plain_call/plain_call.rs"]
mod plain_call;

#[path = "reasoning_levels/reasoning_levels.rs"]
mod reasoning_levels;

#[path = "thinking_tool_round/thinking_tool_round.rs"]
mod thinking_tool_round;

#[path = "structured_truncation/structured_truncation.rs"]
mod structured_truncation;

#[path = "error_messages/error_messages.rs"]
mod error_messages;

#[path = "custom_provider/custom_provider.rs"]
mod custom_provider;

#[path = "check_batch/check_batch.rs"]
mod check_batch;

#[path = "live_call/live_call.rs"]
mod live_call;

#[path = "live_thinking/live_thinking.rs"]
mod live_thinking;

#[path = "live_image/live_image.rs"]
mod live_image;

#[path = "live_own_error/live_own_error.rs"]
mod live_own_error;

#[path = "cut_tool_call/cut_tool_call.rs"]
mod cut_tool_call;

#[path = "logprobs_asked/logprobs_asked.rs"]
mod logprobs_asked;

#[path = "logprobs_malformed/logprobs_malformed.rs"]
mod logprobs_malformed;
