//! From the answer of `POST /v1/responses` to a `ModelResponse`.

use serde_json::Value;

use super::OpenAi;
use crate::models::error::fixed_code;
use crate::models::{ModelError, ModelResponse, ResponseInfo, StopReason, ToolCall, Usage};

fn count(v: Option<&Value>, key: &str) -> u64 {
    v.and_then(|v| v.get(key)).and_then(Value::as_u64).unwrap_or(0)
}

impl OpenAi {
    pub(crate) fn parse_response(
        &self,
        schema: bool,
        v: &Value,
        execution_time: f64,
    ) -> Result<ModelResponse, ModelError> {
        let status = v.get("status").and_then(Value::as_str).unwrap_or("completed");
        if status == "failed" {
            let code = v.pointer("/error/code").and_then(Value::as_str);
            let said = v.pointer("/error/message").and_then(Value::as_str).map(str::to_string);
            return Err(self.who.provider_said(None, code, "the response failed", said));
        }
        let items = v
            .get("output")
            .and_then(Value::as_array)
            .ok_or_else(|| self.who.invalid("no `output` array"))?;
        let incomplete = v.pointer("/incomplete_details/reason").and_then(Value::as_str);
        let mut stop_reason = match (status, incomplete) {
            ("incomplete", Some("content_filter")) => StopReason::Refused,
            ("incomplete", _) => StopReason::MaxTokens,
            ("completed", _) => StopReason::End,
            ("cancelled", _) => StopReason::Other,
            (other, _) => {
                return Err(match fixed_code(Some(other)) {
                    Some(kept) => self.who.invalid(format!("unexpected status `{kept}`")),
                    None => self.who.invalid_said("an unexpected status", other),
                });
            }
        };
        // a refusal part may come after a tool call: it is looked for first
        let refused = items
            .iter()
            .filter(|i| i.get("type").and_then(Value::as_str) == Some("message"))
            .flat_map(|i| i.get("content").and_then(Value::as_array).into_iter().flatten())
            .any(|p| {
                p.get("type").and_then(Value::as_str) == Some("refusal")
                    && !p.get("refusal").and_then(Value::as_str).unwrap_or_default().is_empty()
            });
        if refused {
            stop_reason = StopReason::Refused;
        }

        let (mut text, mut refusal) = (String::new(), String::new());
        let mut reasoning = Vec::new();
        let mut tools_call = Vec::new();
        for item in items {
            match item.get("type").and_then(Value::as_str) {
                // opaque: the tool loop sends them back as they are
                Some("reasoning") => reasoning.push(item.clone()),
                Some("message") => {
                    for part in item.get("content").and_then(Value::as_array).into_iter().flatten() {
                        match part.get("type").and_then(Value::as_str) {
                            Some("output_text") => {
                                text.push_str(part.get("text").and_then(Value::as_str).unwrap_or_default())
                            }
                            Some("refusal") => {
                                refusal.push_str(part.get("refusal").and_then(Value::as_str).unwrap_or_default())
                            }
                            _ => {}
                        }
                    }
                }
                // a refusal asks for nothing: its tool calls are never run
                Some("function_call") if stop_reason == StopReason::Refused => {}
                // a cut answer may hold a tool call cut in the middle
                Some("function_call") if stop_reason.is_cut() => return Err(self.who.truncated()),
                Some("function_call") => {
                    // the arguments are a JSON string
                    let raw = item
                        .get("arguments")
                        .and_then(Value::as_str)
                        .ok_or_else(|| self.who.invalid("a `function_call` without `arguments`"))?;
                    let arguments = serde_json::from_str::<Value>(raw)
                        .ok()
                        .filter(Value::is_object)
                        .ok_or_else(|| self.who.invalid("tool call arguments are not a JSON object"))?;
                    tools_call.push(ToolCall {
                        call_id: item
                            .get("call_id")
                            .and_then(Value::as_str)
                            .ok_or_else(|| self.who.invalid("a `function_call` without `call_id`"))?
                            .to_string(),
                        function_name: item
                            .get("name")
                            .and_then(Value::as_str)
                            .ok_or_else(|| self.who.invalid("a `function_call` without `name`"))?
                            .to_string(),
                        arguments,
                        // the whole item goes back with the result
                        opaque: Some(item.clone()),
                    });
                }
                _ => {}
            }
        }

        if !tools_call.is_empty() {
            stop_reason = StopReason::ToolCall;
        }

        let (mut message, mut structured) = (None, None);
        if !refusal.is_empty() {
            message = Some(refusal);
        } else if schema {
            if stop_reason.is_cut() {
                return Err(self.who.truncated());
            }
            if tools_call.is_empty() && stop_reason != StopReason::Refused {
                structured = Some(
                    serde_json::from_str(&text)
                        .map_err(|e| self.who.invalid(format!("the output is not the JSON asked for: {e}")))?,
                );
            }
        } else if !text.is_empty() {
            message = Some(text);
        }

        let u = v.get("usage");
        let cached = count(u.and_then(|u| u.get("input_tokens_details")), "cached_tokens");
        let usage = Usage {
            input_tokens: count(u, "input_tokens").saturating_sub(cached),
            cache_read_tokens: cached,
            cache_write_tokens: count(u.and_then(|u| u.get("input_tokens_details")), "cache_write_tokens"),
            output_tokens: count(u, "output_tokens"),
            reasoning_tokens: count(u.and_then(|u| u.get("output_tokens_details")), "reasoning_tokens"),
        };
        Ok(ModelResponse {
            message,
            structured,
            media: Vec::new(),
            tools_call,
            reasoning,
            stop_reason,
            stop_detail: Some(
                incomplete
                    .map(|r| format!("{status}: {r}"))
                    .unwrap_or_else(|| status.to_string()),
            ),
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
