//! From the answer of `POST /v1/messages` to a `ModelResponse`.

use serde_json::Value;

use super::Anthropic;
use crate::models::{ModelError, ModelResponse, ResponseInfo, StopReason, ToolCall, Usage};

fn count(v: &Value, key: &str) -> u64 {
    v.get(key).and_then(Value::as_u64).unwrap_or(0)
}

impl Anthropic {
    pub(crate) fn parse_message(&self, v: &Value, execution_time: f64) -> Result<ModelResponse, ModelError> {
        let blocks = v
            .get("content")
            .and_then(Value::as_array)
            .ok_or_else(|| self.invalid("no `content` array"))?;
        let stop = v
            .get("stop_reason")
            .and_then(Value::as_str)
            .ok_or_else(|| self.invalid("no `stop_reason`"))?;
        let mut stop_reason = match stop {
            "end_turn" | "stop_sequence" => StopReason::End,
            "tool_use" => StopReason::ToolCall,
            "max_tokens" => StopReason::MaxTokens,
            "model_context_window_exceeded" => StopReason::ContextExceeded,
            "refusal" => StopReason::Refused,
            _ => StopReason::Other,
        };

        let mut text = String::new();
        let mut reasoning = Vec::new();
        let mut tools_call = Vec::new();
        for b in blocks {
            match b.get("type").and_then(Value::as_str) {
                Some("text") => text.push_str(b.get("text").and_then(Value::as_str).unwrap_or_default()),
                // opaque: the tool loop sends them back as they are
                Some("thinking" | "redacted_thinking") => reasoning.push(b.clone()),
                // a refusal asks for nothing: its tool calls are never run
                Some("tool_use") if stop_reason == StopReason::Refused => {}
                Some("tool_use") => {
                    // a cut answer may hold a `tool_use` block cut in the middle
                    if stop_reason.is_cut() {
                        return Err(self.who.truncated(stop_reason));
                    }
                    let arguments = b
                        .get("input")
                        .filter(|i| i.is_object())
                        .cloned()
                        .ok_or_else(|| self.invalid("a `tool_use` block without an object `input`"))?;
                    tools_call.push(ToolCall {
                        call_id: b
                            .get("id")
                            .and_then(Value::as_str)
                            .ok_or_else(|| self.invalid("a `tool_use` block without `id`"))?
                            .to_string(),
                        function_name: b
                            .get("name")
                            .and_then(Value::as_str)
                            .ok_or_else(|| self.invalid("a `tool_use` block without `name`"))?
                            .to_string(),
                        arguments,
                        opaque: None,
                    });
                }
                _ => {}
            }
        }

        if !tools_call.is_empty() {
            stop_reason = StopReason::ToolCall;
        }

        // with a schema or not, the text goes back as it came: the JSON is read by `models::structured`
        let (message, structured) = (Some(text).filter(|t| !t.is_empty()), None);

        let usage_v = v.get("usage").cloned().unwrap_or(Value::Null);
        let usage = Usage {
            input_tokens: count(&usage_v, "input_tokens"),
            cache_read_tokens: count(&usage_v, "cache_read_input_tokens"),
            cache_write_tokens: count(&usage_v, "cache_creation_input_tokens"),
            output_tokens: count(&usage_v, "output_tokens"),
            reasoning_tokens: usage_v
                .pointer("/output_tokens_details/thinking_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0),
        };
        Ok(ModelResponse {
            message,
            structured,
            media: Vec::new(),
            tools_call,
            reasoning,
            logprobs: Vec::new(),
            stop_reason,
            stop_detail: Some(stop.to_string()),
            info: ResponseInfo {
                provider: self.who.provider.into(),
                model: v
                    .get("model")
                    .and_then(Value::as_str)
                    .unwrap_or(&self.model)
                    .to_string(),
                execution_time,
                usage,
                corrections: None,
            },
        })
    }
}
