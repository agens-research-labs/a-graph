//! From a `ModelInvocation` to the body of `POST /api/chat`.

use serde_json::{Map, Value, json};

use super::Ollama;
use crate::models::Reasoning;
use crate::models::{MediaInput, ModelError, ModelInvocation, Role, ToolRound};

impl Ollama {
    pub(crate) fn request_body(&self, call: &ModelInvocation) -> Result<Value, ModelError> {
        if !call.output_media.is_empty() {
            return Err(self.who.unsupported(vec!["this server produces text only".into()]));
        }
        let mut body = Map::new();
        body.insert("model".into(), json!(self.model));
        body.insert("stream".into(), json!(false));
        body.insert("messages".into(), Value::Array(self.messages(call)));
        if !call.tools.is_empty() {
            let tools: Vec<Value> = call.tools.iter().map(|t| json!({ "type": "function", "function": { "name": t.name, "description": t.description, "parameters": t.parameters } })).collect();
            body.insert("tools".into(), Value::Array(tools));
        }
        if let Some(schema) = &call.schema {
            body.insert("format".into(), schema.clone());
        }
        // `off` is false; the levels are the strings models like gpt-oss take, the nearest one
        // where a level is missing. What a model does with them is the model's business.
        match call.reasoning {
            None => {}
            Some(Reasoning::Off) => {
                body.insert("think".into(), json!(false));
            }
            Some(Reasoning::Minimal | Reasoning::Low) => {
                body.insert("think".into(), json!("low"));
            }
            Some(Reasoning::Medium) => {
                body.insert("think".into(), json!("medium"));
            }
            Some(Reasoning::High | Reasoning::Max) => {
                body.insert("think".into(), json!("high"));
            }
        }
        let mut options = Map::new();
        if let Some(t) = call.temperature {
            options.insert("temperature".into(), json!(t));
        }
        if let Some(p) = call.top_p {
            options.insert("top_p".into(), json!(p));
        }
        if let Some(n) = call.max_tokens {
            options.insert("num_predict".into(), json!(n));
        }
        if !options.is_empty() {
            body.insert("options".into(), Value::Object(options));
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

        // markdown media go before the question, as text; images go in `images`
        let mut parts: Vec<&str> = Vec::new();
        let mut images = Vec::new();
        for m in &call.media {
            match m {
                MediaInput::Markdown(md) => parts.push(md),
                MediaInput::Image { data, .. } => images.push(json!(data)),
            }
        }
        parts.push(&call.user);
        let mut user = json!({ "role": "user", "content": parts.join("\n\n") });
        if !images.is_empty() {
            user["images"] = Value::Array(images);
        }
        messages.push(user);

        for round in &call.tool_rounds {
            messages.extend(tool_round(round));
        }
        messages
    }
}

/// The assistant turn that asked for the tools, echoed as it came, then one `tool` message per result.
fn tool_round(round: &ToolRound) -> Vec<Value> {
    let mut assistant = json!({ "role": "assistant", "content": round.message.clone().unwrap_or_default() });
    let thinking: Vec<&str> = round.reasoning.iter().filter_map(|b| b.get("thinking").and_then(Value::as_str)).collect();
    if !thinking.is_empty() {
        assistant["thinking"] = json!(thinking.join("\n"));
    }
    let calls: Vec<Value> = round.calls.iter().map(|c| json!({ "id": c.call_id, "function": { "name": c.function_name, "arguments": c.arguments } })).collect();
    if !calls.is_empty() {
        assistant["tool_calls"] = Value::Array(calls);
    }

    let mut messages = vec![assistant];
    for r in &round.results {
        let name = round.calls.iter().find(|c| c.call_id == r.call_id).map(|c| c.function_name.clone()).unwrap_or_default();
        let text = match &r.result {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        // the contract has no error flag: the text says it
        let content = if r.is_error { format!("error: {text}") } else { text };
        messages.push(json!({ "role": "tool", "tool_name": name, "content": content }));
    }
    messages
}
