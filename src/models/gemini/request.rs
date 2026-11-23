//! From a `ModelInvocation` to the body of `POST /v1beta/interactions`.

use serde_json::{Map, Value, json};

use super::Gemini;
use super::media::image_mime;
use crate::models::Reasoning;
use crate::models::{ChatEntry, MediaInput, ModelError, ModelInvocation, Role, ToolRound};

fn text_item(text: &str) -> Value {
    json!({ "type": "text", "text": text })
}

impl Gemini {
    pub(crate) fn request_body(&self, call: &ModelInvocation) -> Result<Value, ModelError> {
        let media = self
            .media_format(&call.output_media)
            .map_err(|p| self.who.unsupported(p))?;
        if media.is_some() && call.schema.is_some() {
            return Err(self.who.unsupported(vec![
                "an output schema and a produced media cannot be asked together".into(),
            ]));
        }

        let mut body = Map::new();
        body.insert("model".into(), json!(self.model));
        body.insert("store".into(), json!(false));
        if !call.system.is_empty() {
            body.insert("system_instruction".into(), json!(call.system));
        }
        body.insert("input".into(), Value::Array(self.steps(call)?));
        if !call.tools.is_empty() {
            let tools: Vec<Value> = call.tools.iter().map(|t| json!({ "type": "function", "name": t.name, "description": t.description, "parameters": t.parameters })).collect();
            body.insert("tools".into(), Value::Array(tools));
        }

        let mut config = Map::new();
        if let Some(n) = call.max_tokens {
            config.insert("max_output_tokens".into(), json!(n));
        }
        if let Some(t) = call.temperature {
            config.insert("temperature".into(), json!(t));
        }
        if let Some(p) = call.top_p {
            config.insert("top_p".into(), json!(p));
        }
        if let Some(level) = call.reasoning {
            let level = match level {
                Reasoning::Off => {
                    return Err(self.who.unsupported(vec![
                        "Gemini cannot switch thinking off: its levels are minimal, low, medium, high".into(),
                    ]));
                }
                Reasoning::Minimal => "minimal",
                Reasoning::Low => "low",
                Reasoning::Medium => "medium",
                // the nearest level it has
                Reasoning::High | Reasoning::Max => "high",
            };
            config.insert("thinking_level".into(), json!(level));
            // without it the thinking text is not returned
            config.insert("thinking_summaries".into(), json!("auto"));
        }

        if let Some(m) = media {
            body.insert("response_format".into(), m.response_format);
            if let Some(s) = m.speech_config {
                config.insert("speech_config".into(), s);
            }
        } else if let Some(schema) = &call.schema {
            body.insert(
                "response_format".into(),
                json!({ "type": "text", "mime_type": "application/json", "schema": schema }),
            );
        }
        if !config.is_empty() {
            body.insert("generation_config".into(), Value::Object(config));
        }
        Ok(Value::Object(body))
    }

    /// The whole conversation as steps: history, the question, then the tool rounds.
    fn steps(&self, call: &ModelInvocation) -> Result<Vec<Value>, ModelError> {
        let entry = |e: &ChatEntry| {
            let kind = match e.role {
                Role::User => "user_input",
                Role::Assistant => "model_output",
            };
            json!({ "type": kind, "content": [text_item(&e.content)] })
        };
        let mut steps: Vec<Value> = call.history.iter().map(entry).collect();

        let mut content = Vec::new();
        for m in &call.media {
            match m {
                MediaInput::Markdown(md) => content.push(text_item(md)),
                MediaInput::Image { format, data } => {
                    content.push(json!({ "type": "image", "mime_type": image_mime(format), "data": data }))
                }
            }
        }
        if !call.user.is_empty() {
            content.push(text_item(&call.user));
        }
        steps.push(json!({ "type": "user_input", "content": content }));

        for round in &call.tool_rounds {
            // the corrections of `schema_prompt` that the model answered with this round, where they happened
            steps.extend(round.corrections.iter().map(entry));
            steps.extend(tool_round(round));
        }
        // the corrections that no tool round followed come last
        steps.extend(call.corrections.iter().map(entry));
        Ok(steps)
    }
}

/// The steps the model produced when it asked for the tools, echoed as they came (the thought
/// steps carry the signatures), then the results.
fn tool_round(round: &ToolRound) -> Vec<Value> {
    let mut steps: Vec<Value> = round.reasoning.clone();
    if let Some(m) = round.message.as_deref().filter(|m| !m.is_empty()) {
        steps.push(json!({ "type": "model_output", "content": [text_item(m)] }));
    }
    for c in &round.calls {
        let mut step =
            json!({ "type": "function_call", "id": c.call_id, "name": c.function_name, "arguments": c.arguments });
        if let Some(sig) = &c.opaque {
            step["signature"] = sig.clone();
        }
        steps.push(step);
    }
    for r in &round.results {
        let name = round
            .calls
            .iter()
            .find(|c| c.call_id == r.call_id)
            .map(|c| c.function_name.clone())
            .unwrap_or_default();
        let text = match &r.result {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        let mut step =
            json!({ "type": "function_result", "name": name, "call_id": r.call_id, "result": [text_item(&text)] });
        if r.is_error {
            step["is_error"] = json!(true);
        }
        steps.push(step);
    }
    steps
}
