//! From a `ModelInvocation` to the body of `POST /v1/responses`.

use serde_json::{Map, Value, json};

use super::OpenAi;
use crate::models::Reasoning;
use crate::models::{ChatEntry, MediaInput, ModelError, ModelInvocation, Role, ToolRound};

/// The media type of an image format, as it goes in a data URL.
fn image_mime(format: &str) -> String {
    match format.to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg".into(),
        other => format!("image/{other}"),
    }
}

impl OpenAi {
    pub(crate) fn request_body(&self, call: &ModelInvocation) -> Result<Value, ModelError> {
        self.request_body_for(&self.model, call)
    }

    /// `model`: the one the body names (a batch on Azure names its own deployment).
    pub(super) fn request_body_for(&self, model: &str, call: &ModelInvocation) -> Result<Value, ModelError> {
        let mut body = Map::new();
        body.insert("model".into(), json!(model));
        // stateless: nothing is stored, the whole conversation is sent every time
        body.insert("store".into(), json!(false));
        if !call.system.is_empty() {
            body.insert("instructions".into(), json!(call.system));
        }
        body.insert("input".into(), Value::Array(self.input(call)));
        if !call.tools.is_empty() {
            let tools: Vec<Value> = call.tools.iter().map(|t| json!({ "type": "function", "name": t.name, "description": t.description, "parameters": t.parameters, "strict": false })).collect();
            body.insert("tools".into(), Value::Array(tools));
        }
        if let Some(schema) = &call.schema {
            body.insert(
                "text".into(),
                json!({ "format": { "type": "json_schema", "name": "output", "schema": schema, "strict": false } }),
            );
        }
        if let Some(level) = call.reasoning {
            let effort = match level {
                Reasoning::Off => "none",
                Reasoning::Minimal => "minimal",
                Reasoning::Low => "low",
                Reasoning::Medium => "medium",
                Reasoning::High => "high",
                Reasoning::Max => "max",
            };
            let mut r = json!({ "effort": effort });
            if level != Reasoning::Off {
                // without it the reasoning text is not returned
                r["summary"] = json!("auto");
            }
            body.insert("reasoning".into(), r);
        }
        let mut include = Vec::new();
        // a stateless tool loop needs the reasoning items back, encrypted
        if call.reasoning.is_some() || !call.tools.is_empty() {
            include.push("reasoning.encrypted_content");
        }
        if let Some(n) = call.logprobs {
            include.push("message.output_text.logprobs");
            if n > 0 {
                body.insert("top_logprobs".into(), json!(n));
            }
        }
        if !include.is_empty() {
            body.insert("include".into(), json!(include));
        }
        if let Some(n) = call.max_tokens {
            body.insert("max_output_tokens".into(), json!(n));
        }
        if let Some(t) = call.temperature {
            body.insert("temperature".into(), json!(t));
        }
        if let Some(p) = call.top_p {
            body.insert("top_p".into(), json!(p));
        }
        Ok(Value::Object(body))
    }

    fn input(&self, call: &ModelInvocation) -> Vec<Value> {
        let entry = |e: &ChatEntry| {
            let role = match e.role {
                Role::User => "user",
                Role::Assistant => "assistant",
            };
            json!({ "role": role, "content": e.content })
        };
        let mut items: Vec<Value> = call.history.iter().map(entry).collect();

        let mut parts = Vec::new();
        for m in &call.media {
            match m {
                MediaInput::Markdown(md) => parts.push(json!({ "type": "input_text", "text": md })),
                MediaInput::Image { format, data } => {
                    parts.push(json!({ "type": "input_image", "image_url": format!("data:{};base64,{data}", image_mime(format)), "detail": "auto" }));
                }
            }
        }
        if !call.user.is_empty() {
            parts.push(json!({ "type": "input_text", "text": call.user }));
        }
        items.push(json!({ "role": "user", "content": parts }));

        for round in &call.tool_rounds {
            items.extend(tool_round(round));
        }
        // the corrections of `schema_prompt` come last, after the tool rounds
        items.extend(call.corrections.iter().map(entry));
        items
    }
}

/// The items the model produced when it asked for the tools, echoed as they came (the function
/// call items are kept whole in `opaque`), then one output per result.
fn tool_round(round: &ToolRound) -> Vec<Value> {
    let mut items: Vec<Value> = round.reasoning.clone();
    if let Some(m) = round.message.as_deref().filter(|m| !m.is_empty()) {
        items.push(json!({ "role": "assistant", "content": m }));
    }
    for c in &round.calls {
        items.push(c.opaque.clone().unwrap_or_else(|| json!({ "type": "function_call", "call_id": c.call_id, "name": c.function_name, "arguments": c.arguments.to_string() })));
    }
    for r in &round.results {
        let text = match &r.result {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        // the output has no error flag: the text says it
        let output = if r.is_error { format!("error: {text}") } else { text };
        items.push(json!({ "type": "function_call_output", "call_id": r.call_id, "output": output }));
    }
    items
}
