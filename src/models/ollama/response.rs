//! From the answer of `POST /api/chat` to a `ModelResponse`.

use serde_json::{Value, json};

use super::Ollama;
use crate::models::{ModelError, ModelResponse, ResponseInfo, StopReason, ToolCall, Usage};

impl Ollama {
    pub(crate) fn parse_chat(&self, schema: bool, v: &Value, execution_time: f64) -> Result<ModelResponse, ModelError> {
        let msg = v.get("message").ok_or_else(|| self.who.invalid("no `message`"))?;
        let text = msg.get("content").and_then(Value::as_str).unwrap_or_default();
        let done_reason = v.get("done_reason").and_then(Value::as_str).unwrap_or_default();
        let mut stop_reason = match done_reason {
            "stop" | "" => StopReason::End,
            "length" => StopReason::MaxTokens,
            _ => StopReason::Other,
        };

        let mut tools_call = Vec::new();
        for (n, c) in msg
            .get("tool_calls")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            // a cut answer may hold a tool call cut in the middle
            if stop_reason.is_cut() {
                return Err(self.who.truncated());
            }
            let f = c
                .get("function")
                .ok_or_else(|| self.who.invalid("a tool call without `function`"))?;
            let arguments = match f.get("arguments") {
                Some(a) if a.is_object() => a.clone(),
                // some servers send the arguments as a JSON string
                Some(Value::String(s)) => serde_json::from_str::<Value>(s)
                    .ok()
                    .filter(Value::is_object)
                    .ok_or_else(|| self.who.invalid("tool call arguments are not a JSON object"))?,
                _ => return Err(self.who.invalid("a tool call without object `arguments`")),
            };
            tools_call.push(ToolCall {
                // the server may give no id: one is made, unique in this answer
                call_id: c
                    .get("id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("call_{n}")),
                function_name: f
                    .get("name")
                    .and_then(Value::as_str)
                    .ok_or_else(|| self.who.invalid("a tool call without `name`"))?
                    .to_string(),
                arguments,
                opaque: None,
            });
        }

        // the thinking text is the only thing to send back
        let reasoning = match msg.get("thinking").and_then(Value::as_str).filter(|t| !t.is_empty()) {
            Some(t) => vec![json!({ "thinking": t })],
            None => Vec::new(),
        };

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
                    serde_json::from_str(text)
                        .map_err(|e| self.who.invalid(format!("the output is not the JSON asked for: {e}")))?,
                );
            }
        } else if !text.is_empty() {
            message = Some(text.to_string());
        }

        // prompt_eval_count is the whole prompt, cached tokens included; the thinking tokens are in
        // eval_count and the server does not count them apart
        let n = |k: &str| v.get(k).and_then(Value::as_u64).unwrap_or(0);
        let cached = n("prompt_eval_cached_count");
        let usage = Usage {
            input_tokens: n("prompt_eval_count").saturating_sub(cached),
            cache_read_tokens: cached,
            cache_write_tokens: 0,
            output_tokens: n("eval_count"),
            reasoning_tokens: 0,
        };

        Ok(ModelResponse {
            message,
            structured,
            media: Vec::new(),
            tools_call,
            reasoning,
            stop_reason,
            stop_detail: Some(done_reason.to_string()),
            info: ResponseInfo {
                provider: self.who.provider.into(),
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
