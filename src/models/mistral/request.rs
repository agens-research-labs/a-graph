//! From a `ModelInvocation` to the body of `POST /v1/chat/completions`.

use serde_json::{Map, Value, json};

use super::Mistral;
use crate::models::Reasoning;
use crate::models::{MediaInput, ModelError, ModelInvocation, Role, ToolRound};

fn image_mime(format: &str) -> String {
    match format.to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg".into(),
        other => format!("image/{other}"),
    }
}

impl Mistral {
    pub(crate) fn request_body(&self, call: &ModelInvocation) -> Result<Value, ModelError> {
        let mut body = Map::new();
        body.insert("model".into(), json!(self.model));
        body.insert("messages".into(), Value::Array(self.messages(call)));
        if !call.tools.is_empty() {
            let tools: Vec<Value> = call.tools.iter().map(|t| json!({ "type": "function", "function": { "name": t.name, "description": t.description, "parameters": t.parameters } })).collect();
            body.insert("tools".into(), Value::Array(tools));
        }
        if let Some(schema) = &call.schema {
            body.insert("response_format".into(), json!({ "type": "json_schema", "json_schema": { "name": "output", "schema": schema, "strict": false } }));
        }
        if let Some(level) = call.reasoning {
            // Mistral has no `max`: its top level is `xhigh`
            let effort = match level {
                Reasoning::Off => "none",
                Reasoning::Minimal => "minimal",
                Reasoning::Low => "low",
                Reasoning::Medium => "medium",
                Reasoning::High => "high",
                Reasoning::Max => "xhigh",
            };
            body.insert("reasoning_effort".into(), json!(effort));
        }
        if let Some(n) = call.max_tokens {
            body.insert("max_tokens".into(), json!(n));
        }
        if let Some(t) = call.temperature {
            body.insert("temperature".into(), json!(t));
        }
        if let Some(p) = call.top_p {
            body.insert("top_p".into(), json!(p));
        }
        Ok(Value::Object(body))
    }

    fn messages(&self, call: &ModelInvocation) -> Vec<Value> {
        let mut messages = Vec::new();
        if !call.system.is_empty() {
            messages.push(json!({ "role": "system", "content": call.system }));
        }
        for e in &call.history {
            let role = match e.role {
                Role::User => "user",
                Role::Assistant => "assistant",
            };
            messages.push(json!({ "role": role, "content": e.content }));
        }

        let mut chunks = Vec::new();
        for m in &call.media {
            match m {
                MediaInput::Markdown(md) => chunks.push(json!({ "type": "text", "text": md })),
                MediaInput::Image { format, data } => chunks.push(json!({ "type": "image_url", "image_url": { "url": format!("data:{};base64,{data}", image_mime(format)) } })),
            }
        }
        if !call.user.is_empty() {
            chunks.push(json!({ "type": "text", "text": call.user }));
        }
        messages.push(json!({ "role": "user", "content": chunks }));

        for round in &call.tool_rounds {
            messages.extend(tool_round(round));
        }
        messages
    }
}

/// The assistant message that asked for the tools (the thinking chunks first, as they came),
/// then one `tool` message per result.
fn tool_round(round: &ToolRound) -> Vec<Value> {
    let mut content: Vec<Value> = round.reasoning.clone();
    if let Some(m) = round.message.as_deref().filter(|m| !m.is_empty()) {
        content.push(json!({ "type": "text", "text": m }));
    }
    let calls: Vec<Value> = round
        .calls
        .iter()
        .map(|c| json!({ "id": c.call_id, "type": "function", "function": { "name": c.function_name, "arguments": c.arguments.to_string() } }))
        .collect();
    let mut messages = vec![json!({ "role": "assistant", "content": content, "tool_calls": calls })];
    for r in &round.results {
        let name = round.calls.iter().find(|c| c.call_id == r.call_id).map(|c| c.function_name.clone()).unwrap_or_default();
        let text = match &r.result {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        // the tool message has no error flag: the text says it
        let content = if r.is_error { format!("error: {text}") } else { text };
        messages.push(json!({ "role": "tool", "tool_call_id": r.call_id, "name": name, "content": content }));
    }
    messages
}
