use crate::common::http::{Req, Resp, serve};
use a_graph::models::*;
use a_graph::resources::{GeminiMedia, Generate, MediaKind, MediaProvider, Moderation, OpenaiMedia, Size};
use a_graph::{Path, Rule};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const KEY_VAR: &str = "A_GRAPH_TEST_OPENAI_KEY";

fn api_model(var: &str) -> ApiModel {
    ApiModel {
        common: Common {
            id: "smart".into(),
            model: "gpt-5".into(),
            input: vec![Modality::Text],
            output: vec![Modality::Text],
        },
        api_key: var.into(),
    }
}

/// `base` is the server; the API lives under `/v1`.
fn client(base: &str) -> OpenAi {
    // SAFETY: a variable only these tests use, set to the same value every time.
    unsafe { std::env::set_var(KEY_VAR, "test-key") };
    OpenAi::new(&api_model(KEY_VAR), &Path::root().key("models").idx(0))
        .unwrap()
        .with_base_url(format!("{base}/v1"))
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

fn response(status: &str, output: Value) -> String {
    json!({
        "id": "resp_1", "object": "response", "status": status, "model": "gpt-5-2026", "output": output,
        "usage": { "input_tokens": 100, "input_tokens_details": { "cached_tokens": 40, "cache_write_tokens": 3 }, "output_tokens": 50, "output_tokens_details": { "reasoning_tokens": 20 }, "total_tokens": 150 }
    })
    .to_string()
}

fn message(text: &str) -> Value {
    json!({ "id": "msg_1", "type": "message", "role": "assistant", "status": "completed", "content": [{ "type": "output_text", "text": text, "annotations": [] }] })
}

fn text_answer(text: &str) -> String {
    response("completed", json!([message(text)]))
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

// --- produced media ------------------------------------------------------------------

fn image(kind: &str, g: Generate, p: Option<MediaProvider>) -> ProducedMedia {
    ProducedMedia {
        kind: MediaKind::Img(kind.into()),
        generate: g,
        provider: p,
    }
}

// --- Azure Foundry, publisher openai -----------------------------------------------------

fn azure_model(credentials: AzureCredentials, batch: Option<&str>) -> AzureModel {
    AzureModel {
        common: Common {
            id: "azure_gpt".into(),
            model: "my-gpt-deployment".into(),
            input: vec![Modality::Text],
            output: vec![Modality::Text],
        },
        publisher: AzurePublisher::Openai,
        resource: "My-Res".into(),
        credentials,
        batch: batch.map(|d| AzureBatch { deployment: d.into() }),
    }
}

fn azure_key() -> AzureCredentials {
    // SAFETY: a variable only these tests use, set to the same value every time.
    unsafe { std::env::set_var("A_GRAPH_TEST_AZURE_OPENAI_KEY", "azure-key") };
    AzureCredentials::ApiKey("A_GRAPH_TEST_AZURE_OPENAI_KEY".into())
}

// --- batch ---------------------------------------------------------------------------------

fn batch_server(
    polls_before_done: usize,
    final_status: &'static str,
) -> (impl Fn(&Req) -> Resp + Send + Sync + 'static, Arc<Mutex<usize>>) {
    let polls = Arc::new(Mutex::new(0usize));
    let seen = polls.clone();
    let ok_line = |id: &str, text: &str| {
        json!({ "id": "batch_req_1", "custom_id": id, "response": { "status_code": 200, "request_id": "r", "body": serde_json::from_str::<Value>(&text_answer(text)).unwrap() }, "error": null }).to_string()
    };
    let out = [
        ok_line("two", "second"),
        ok_line("one", "first"),
        json!({ "custom_id": "three", "response": { "status_code": 400, "request_id": "r", "body": { "error": { "message": "bad params" } } }, "error": null }).to_string(),
        json!({ "custom_id": "four", "response": null, "error": { "code": "batch_expired", "message": "the request expired" } }).to_string(),
    ]
    .join("\n");
    let handler = move |req: &Req| match (req.method.as_str(), req.path.as_str()) {
        ("POST", "/v1/files") => Resp::ok(json!({ "id": "file-in", "object": "file" }).to_string()),
        ("POST", "/v1/batches") => Resp::ok(json!({ "id": "batch_1", "status": "validating" }).to_string()),
        ("GET", "/v1/batches/batch_1") => {
            let mut n = polls.lock().unwrap();
            *n += 1;
            let status = if *n > polls_before_done {
                final_status
            } else {
                "in_progress"
            };
            let output = if final_status == "completed" {
                json!("file-out")
            } else {
                Value::Null
            };
            Resp::ok(json!({ "id": "batch_1", "status": status, "output_file_id": output, "error_file_id": null, "errors": { "data": [{ "message": "the batch was invalid" }] } }).to_string())
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

#[path = "tool_item/tool_item.rs"]
mod tool_item;

#[path = "structured_refusals/structured_refusals.rs"]
mod structured_refusals;

#[path = "error_messages/error_messages.rs"]
mod error_messages;

#[path = "image_out/image_out.rs"]
mod image_out;

#[path = "image_url_no_key/image_url_no_key.rs"]
mod image_url_no_key;

#[path = "speech_out/speech_out.rs"]
mod speech_out;

#[path = "check_limits/check_limits.rs"]
mod check_limits;

#[path = "key_from_env/key_from_env.rs"]
mod key_from_env;

#[path = "azure_api_key/azure_api_key.rs"]
mod azure_api_key;

#[path = "azure_ambient/azure_ambient.rs"]
mod azure_ambient;

#[path = "azure_key_env/azure_key_env.rs"]
mod azure_key_env;

#[path = "batch_flow/batch_flow.rs"]
mod batch_flow;

#[path = "failed_batch/failed_batch.rs"]
mod failed_batch;

#[path = "unfit_alone/unfit_alone.rs"]
mod unfit_alone;

#[path = "batch_cancel/batch_cancel.rs"]
mod batch_cancel;

#[path = "batch_retry/batch_retry.rs"]
mod batch_retry;
