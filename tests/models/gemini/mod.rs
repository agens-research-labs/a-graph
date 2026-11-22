use crate::common::http::{Req, Resp, serve};
use a_graph::models::*;
use a_graph::resources::{GeminiImageSize, GeminiMedia, Generate, MediaKind, MediaProvider, OpenaiMedia, Size};
use a_graph::{Path, Rule};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

const KEY_VAR: &str = "A_GRAPH_TEST_GEMINI_KEY";

fn api_model(var: &str) -> ApiModel {
    ApiModel {
        common: Common {
            id: "flash".into(),
            model: "gemini-3.5-flash-lite".into(),
            input: vec![Modality::Text],
            output: vec![Modality::Text],
        },
        api_key: var.into(),
    }
}

fn client(base: &str) -> Gemini {
    // SAFETY: a variable only these tests use, set to the same value every time.
    unsafe { std::env::set_var(KEY_VAR, "test-key") };
    Gemini::new(&api_model(KEY_VAR), &Path::root().key("models").idx(0))
        .unwrap()
        .with_base_url(base)
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
        corrections: vec![],
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

fn interaction(status: &str, steps: Value) -> String {
    json!({
        "status": status, "object": "interaction", "model": "gemini-3.5-flash-lite", "steps": steps,
        "usage": { "total_tokens": 621, "total_input_tokens": 13, "total_cached_tokens": 5, "total_output_tokens": 131, "total_tool_use_tokens": 0, "total_thought_tokens": 477 }
    })
    .to_string()
}

fn output(text: &str) -> Value {
    json!({ "type": "model_output", "content": [{ "type": "text", "text": text }] })
}

fn text_answer(text: &str) -> String {
    interaction(
        "completed",
        json!([{ "type": "thought", "signature": "sig1" }, output(text)]),
    )
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

// --- live (run with: cargo test -- --ignored) ------------------------------------

fn live(model: &str) -> Gemini {
    let mut m = api_model("GEMINI_API_KEY");
    m.common.model = model.into();
    Gemini::new(&m, &Path::root()).unwrap()
}

// --- Gemini on Vertex AI (publisher google) ------------------------------------------------

fn vertex_model(location: &str) -> VertexModel {
    VertexModel {
        common: Common {
            id: "gemini_gcp".into(),
            model: "gemini-3.5-flash-lite".into(),
            input: vec![Modality::Text],
            output: vec![Modality::Text],
        },
        publisher: VertexPublisher::Google,
        project: "my-project".into(),
        location: location.into(),
        batch: None,
    }
}

#[path = "plain_call/plain_call.rs"]
mod plain_call;

#[path = "reasoning_levels/reasoning_levels.rs"]
mod reasoning_levels;

#[path = "tool_signature/tool_signature.rs"]
mod tool_signature;

#[path = "structured_truncation/structured_truncation.rs"]
mod structured_truncation;

#[path = "image_out/image_out.rs"]
mod image_out;

#[path = "audio_out/audio_out.rs"]
mod audio_out;

#[path = "check_limits/check_limits.rs"]
mod check_limits;

#[path = "batch_refused/batch_refused.rs"]
mod batch_refused;

#[path = "error_messages/error_messages.rs"]
mod error_messages;

#[path = "key_from_env/key_from_env.rs"]
mod key_from_env;

#[path = "live_thinking/live_thinking.rs"]
mod live_thinking;

#[path = "live_structured/live_structured.rs"]
mod live_structured;

#[path = "live_tool_round/live_tool_round.rs"]
mod live_tool_round;

#[path = "live_gone_model/live_gone_model.rs"]
mod live_gone_model;

#[path = "live_image/live_image.rs"]
mod live_image;

#[path = "live_audio/live_audio.rs"]
mod live_audio;

#[path = "vertex_mapping/vertex_mapping.rs"]
mod vertex_mapping;

#[path = "cut_tool_call/cut_tool_call.rs"]
mod cut_tool_call;

#[path = "status_not_fixed/status_not_fixed.rs"]
mod status_not_fixed;

#[path = "no_logprobs/no_logprobs.rs"]
mod no_logprobs;

#[path = "correction_last/correction_last.rs"]
mod correction_last;

#[path = "live_correction/live_correction.rs"]
mod live_correction;
