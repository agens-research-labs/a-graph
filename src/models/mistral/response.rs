//! From the answer of `POST /v1/chat/completions` to a `ModelResponse`.

use serde_json::Value;

use super::{Mistral, PROVIDER};
use crate::models::{ModelError, ModelResponse, ResponseInfo, StopReason, ToolCall, Usage};

impl Mistral {
    pub(crate) fn parse_completion(
        &self,
        schema: bool,
        v: &Value,
        execution_time: f64,
    ) -> Result<ModelResponse, ModelError> {
        let choice = v
            .pointer("/choices/0")
            .ok_or_else(|| self.who.invalid("no `choices`"))?;
        let msg = choice.get("message").ok_or_else(|| self.who.invalid("no `message`"))?;
        let finish = choice.get("finish_reason").and_then(Value::as_str).unwrap_or_default();
        let mut stop_reason = match finish {
            "stop" => StopReason::End,
            "length" => StopReason::MaxTokens,
            "model_length" => StopReason::ContextExceeded,
            "tool_calls" => StopReason::ToolCall,
            "error" => return Err(self.who.provider_error(None, "the model stopped with an error")),
            _ => StopReason::Other,
        };

        // the content is a string, or chunks: text and thinking
        let mut text = String::new();
        let mut reasoning = Vec::new();
        match msg.get("content") {
            Some(Value::String(s)) => text.push_str(s),
            Some(Value::Array(chunks)) => {
                for c in chunks {
                    match c.get("type").and_then(Value::as_str) {
                        Some("text") => text.push_str(c.get("text").and_then(Value::as_str).unwrap_or_default()),
                        // opaque: the tool loop sends them back as they are
                        Some("thinking") => reasoning.push(c.clone()),
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        let mut tools_call = Vec::new();
        for c in msg.get("tool_calls").and_then(Value::as_array).into_iter().flatten() {
            // a cut answer may hold a tool call cut in the middle
            if stop_reason.is_cut() {
                return Err(self.who.truncated());
            }
            let f = c
                .get("function")
                .ok_or_else(|| self.who.invalid("a tool call without `function`"))?;
            // the arguments are a JSON string, or already an object
            let arguments = match f.get("arguments") {
                Some(a) if a.is_object() => a.clone(),
                Some(Value::String(s)) => serde_json::from_str::<Value>(s)
                    .ok()
                    .filter(Value::is_object)
                    .ok_or_else(|| self.who.invalid("tool call arguments are not a JSON object"))?,
                _ => return Err(self.who.invalid("a tool call without `arguments`")),
            };
            tools_call.push(ToolCall {
                call_id: c
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| self.who.invalid("a tool call without `id`"))?
                    .to_string(),
                function_name: f
                    .get("name")
                    .and_then(Value::as_str)
                    .ok_or_else(|| self.who.invalid("a tool call without `name`"))?
                    .to_string(),
                arguments,
                opaque: None,
            });
        }

        if !tools_call.is_empty() {
            stop_reason = StopReason::ToolCall;
        }

        let (mut message, mut structured) = (None, None);
        if schema {
            if stop_reason.is_cut() {
                return Err(self.who.truncated());
            }
            if tools_call.is_empty() {
                structured = Some(
                    serde_json::from_str(&text)
                        .map_err(|e| self.who.invalid(format!("the output is not the JSON asked for: {e}")))?,
                );
            }
        } else if !text.is_empty() {
            message = Some(text);
        }

        let n = |p: &str| v.pointer(p).and_then(Value::as_u64).unwrap_or(0);
        let usage = Usage {
            input_tokens: n("/usage/prompt_tokens"),
            cache_read_tokens: 0,
            cache_write_tokens: 0,
            output_tokens: n("/usage/completion_tokens"),
            reasoning_tokens: n("/usage/completion_tokens_details/reasoning_tokens"),
        };
        Ok(ModelResponse {
            message,
            structured,
            media: Vec::new(),
            tools_call,
            reasoning,
            stop_reason,
            stop_detail: Some(finish.to_string()),
            info: ResponseInfo {
                provider: PROVIDER.into(),
                model: v
                    .get("model")
                    .and_then(Value::as_str)
                    .unwrap_or(&self.model)
                    .to_string(),
                execution_time,
                usage,
            },
        })
    }
}
