//! From a `ModelInvocation` to the body of `POST /v1/messages`.

use serde_json::{Map, Value, json};

use super::{Anthropic, Flavor};
use crate::models::{MediaInput, ModelError, ModelInvocation, Reasoning, Role, ToolRound};

/// The media type of an image format, if Anthropic accepts it.
pub(super) fn image_mime(format: &str) -> Option<&'static str> {
    match format.to_lowercase().as_str() {
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

fn role(r: Role) -> &'static str {
    match r {
        Role::User => "user",
        Role::Assistant => "assistant",
    }
}

fn text_block(text: &str) -> Value {
    json!({ "type": "text", "text": text })
}

impl Anthropic {
    pub(crate) fn request_body(&self, call: &ModelInvocation) -> Result<Value, ModelError> {
        if !call.output_media.is_empty() {
            return Err(self.unsupported(vec!["anthropic produces text only".into()]));
        }
        let max_tokens = call
            .max_tokens
            .ok_or_else(|| self.unsupported(vec!["max_tokens is mandatory for anthropic".into()]))?;
        let mut body = Map::new();
        body.insert("model".into(), json!(self.model));
        body.insert("max_tokens".into(), json!(max_tokens));
        if !call.system.is_empty() {
            body.insert("system".into(), json!(call.system));
        }
        body.insert("messages".into(), Value::Array(self.messages(call)?));
        if !call.tools.is_empty() {
            let tools: Vec<Value> = call
                .tools
                .iter()
                .map(|t| json!({ "name": t.name, "description": t.description, "input_schema": t.parameters }))
                .collect();
            body.insert("tools".into(), Value::Array(tools));
        }
        if let Some(t) = call.temperature {
            body.insert("temperature".into(), json!(t));
        }
        if let Some(p) = call.top_p {
            body.insert("top_p".into(), json!(p));
        }

        let mut output_config = Map::new();
        match call.reasoning {
            None => {}
            Some(Reasoning::Off) => {
                body.insert("thinking".into(), json!({ "type": "disabled" }));
            }
            Some(level) => {
                // `summarized`: without it newer models return the thinking blocks empty
                body.insert(
                    "thinking".into(),
                    json!({ "type": "adaptive", "display": "summarized" }),
                );
                let effort = match level {
                    Reasoning::Minimal | Reasoning::Low => "low",
                    Reasoning::Medium => "medium",
                    Reasoning::High => "high",
                    Reasoning::Max => "max",
                    Reasoning::Off => unreachable!(),
                };
                output_config.insert("effort".into(), json!(effort));
            }
        }
        if let Some(schema) = &call.schema {
            output_config.insert("format".into(), json!({ "type": "json_schema", "schema": schema }));
        }
        if !output_config.is_empty() {
            body.insert("output_config".into(), Value::Object(output_config));
        }
        if self.flavor == Flavor::Vertex {
            // the model is in the URL, the version in the body
            body.remove("model");
            body.insert("anthropic_version".into(), json!("vertex-2023-10-16"));
        }
        Ok(Value::Object(body))
    }

    fn messages(&self, call: &ModelInvocation) -> Result<Vec<Value>, ModelError> {
        let mut messages: Vec<Value> = call
            .history
            .iter()
            .map(|e| json!({ "role": role(e.role), "content": e.content }))
            .collect();

        // media first, then the question
        let mut content = Vec::new();
        let mut problems = Vec::new();
        for m in &call.media {
            match m {
                MediaInput::Markdown(md) => content.push(text_block(md)),
                MediaInput::Image { format, data } => match image_mime(format) {
                    Some(mime) => content.push(
                        json!({ "type": "image", "source": { "type": "base64", "media_type": mime, "data": data } }),
                    ),
                    None => problems.push(format!(
                        "image format `{format}` is not accepted (jpeg, png, gif, webp)"
                    )),
                },
            }
        }
        if !problems.is_empty() {
            return Err(self.unsupported(problems));
        }
        if !call.user.is_empty() {
            content.push(text_block(&call.user));
        }
        messages.push(json!({ "role": "user", "content": content }));

        for round in &call.tool_rounds {
            messages.extend(tool_round(round));
        }
        Ok(messages)
    }
}

/// The assistant turn that asked for the tools, echoed as it came, then the results.
fn tool_round(round: &ToolRound) -> [Value; 2] {
    let mut assistant: Vec<Value> = round.reasoning.clone();
    if let Some(m) = round.message.as_deref().filter(|m| !m.is_empty()) {
        assistant.push(text_block(m));
    }
    for c in &round.calls {
        assistant.push(json!({ "type": "tool_use", "id": c.call_id, "name": c.function_name, "input": c.arguments }));
    }
    let results: Vec<Value> = round
        .results
        .iter()
        .map(|r| {
            let content = match &r.result {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            json!({ "type": "tool_result", "tool_use_id": r.call_id, "content": content, "is_error": r.is_error })
        })
        .collect();
    [
        json!({ "role": "assistant", "content": assistant }),
        json!({ "role": "user", "content": results }),
    ]
}
