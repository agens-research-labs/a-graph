//! From the answer of the Interactions API to a `ModelResponse`.

use serde_json::Value;

use super::Gemini;
use crate::models::error::fixed_code;
use crate::models::{
    MediaOutput, ModelError, ModelInvocation, ModelResponse, ResponseInfo, StopReason, ToolCall, Usage,
};

fn count(v: &Value, key: &str) -> u64 {
    v.get(key).and_then(Value::as_u64).unwrap_or(0)
}

impl Gemini {
    pub(crate) fn parse_interaction(
        &self,
        call: &ModelInvocation,
        v: &Value,
        execution_time: f64,
    ) -> Result<ModelResponse, ModelError> {
        let status = v
            .get("status")
            .and_then(Value::as_str)
            .ok_or_else(|| self.who.invalid("no `status`"))?;
        if status == "failed" {
            let code = v.pointer("/errors/0/code").and_then(Value::as_str);
            let said = v
                .pointer("/errors/0/message")
                .and_then(Value::as_str)
                .map(str::to_string);
            return Err(self.who.provider_said(None, code, "the interaction failed", said));
        }
        let steps = v
            .get("steps")
            .and_then(Value::as_array)
            .ok_or_else(|| self.who.invalid("no `steps` array"))?;
        let mut stop_reason = match status {
            "completed" => StopReason::End,
            "requires_action" => StopReason::ToolCall,
            // the output reached max_output_tokens (thinking tokens count in it)
            "incomplete" => StopReason::MaxTokens,
            "cancelled" => StopReason::Other,
            other => {
                return Err(match fixed_code(Some(other)) {
                    Some(kept) => self.who.invalid(format!("unexpected status `{kept}`")),
                    None => self.who.invalid_said("an unexpected status", other),
                });
            }
        };
        let expected = self
            .media_format(&call.output_media)
            .ok()
            .flatten()
            .map(|m| m.mime_type);

        let mut text = String::new();
        let mut reasoning = Vec::new();
        let mut media = Vec::new();
        let mut tools_call = Vec::new();
        for step in steps {
            match step.get("type").and_then(Value::as_str) {
                // opaque: the tool loop sends them back as they are
                Some("thought") => reasoning.push(step.clone()),
                Some("model_output") => {
                    for item in step.get("content").and_then(Value::as_array).into_iter().flatten() {
                        match item.get("type").and_then(Value::as_str) {
                            Some("text") => text.push_str(item.get("text").and_then(Value::as_str).unwrap_or_default()),
                            Some(kind @ ("image" | "audio")) => {
                                let data = item
                                    .get("data")
                                    .and_then(Value::as_str)
                                    .ok_or_else(|| self.who.invalid(format!("a {kind} item without `data`")))?;
                                // an audio item does not say its type: it is the one asked for
                                let mime = item
                                    .get("mime_type")
                                    .and_then(Value::as_str)
                                    .map(str::to_string)
                                    .or_else(|| expected.clone());
                                let mime =
                                    mime.ok_or_else(|| self.who.invalid(format!("a {kind} item and no media asked")))?;
                                if let Some(want) = &expected
                                    && !mime.eq_ignore_ascii_case(want)
                                {
                                    return Err(self.who.mime_mismatch(want, &mime));
                                }
                                media.push(MediaOutput {
                                    mime_type: mime,
                                    data: data.to_string(),
                                    id: None,
                                });
                            }
                            _ => {}
                        }
                    }
                }
                // a cut answer may hold a tool call cut in the middle
                Some("function_call") if stop_reason.is_cut() => return Err(self.who.truncated()),
                Some("function_call") => tools_call.push(ToolCall {
                    call_id: step
                        .get("id")
                        .and_then(Value::as_str)
                        .ok_or_else(|| self.who.invalid("a `function_call` without `id`"))?
                        .to_string(),
                    function_name: step
                        .get("name")
                        .and_then(Value::as_str)
                        .ok_or_else(|| self.who.invalid("a `function_call` without `name`"))?
                        .to_string(),
                    arguments: step
                        .get("arguments")
                        .filter(|a| a.is_object())
                        .cloned()
                        .ok_or_else(|| self.who.invalid("a `function_call` without object `arguments`"))?,
                    opaque: step.get("signature").cloned(),
                }),
                _ => {}
            }
        }

        if !tools_call.is_empty() {
            stop_reason = StopReason::ToolCall;
        }

        let (mut message, mut structured) = (None, None);
        if call.schema.is_some() {
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

        // total_output_tokens does not include the thinking tokens; total_input_tokens includes the cached ones
        let u = v.get("usage").cloned().unwrap_or(Value::Null);
        let (input, cached, output, thought) = (
            count(&u, "total_input_tokens"),
            count(&u, "total_cached_tokens"),
            count(&u, "total_output_tokens"),
            count(&u, "total_thought_tokens"),
        );
        let usage = Usage {
            input_tokens: input.saturating_sub(cached),
            cache_read_tokens: cached,
            cache_write_tokens: 0,
            output_tokens: output + thought,
            reasoning_tokens: thought,
        };

        Ok(ModelResponse {
            message,
            structured,
            media,
            tools_call,
            reasoning,
            stop_reason,
            stop_detail: Some(status.to_string()),
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
