//! From a `Converse` answer to a `ModelResponse`, and the SDK's errors to ours.

use aws_sdk_bedrockruntime::error::{ProvideErrorMetadata, SdkError};
use aws_sdk_bedrockruntime::operation::converse::ConverseOutput;
use aws_sdk_bedrockruntime::types::{
    ContentBlock, ConverseOutput as Output, ReasoningContentBlock, StopReason as BedrockStop,
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::json;

use super::Bedrock;
use super::convert::from_document;
use crate::models::{ModelError, ModelResponse, ResponseInfo, StopReason, ToolCall, Usage};

impl Bedrock {
    pub(super) fn parse_converse(
        &self,
        schema: bool,
        out: ConverseOutput,
        execution_time: f64,
    ) -> Result<ModelResponse, ModelError> {
        let Some(Output::Message(message)) = out.output() else {
            return Err(self.who.invalid("no message in the answer"));
        };
        let stop = out.stop_reason();
        let mut stop_reason = match stop {
            BedrockStop::EndTurn | BedrockStop::StopSequence => StopReason::End,
            BedrockStop::ToolUse => StopReason::ToolCall,
            BedrockStop::MaxTokens => StopReason::MaxTokens,
            BedrockStop::ModelContextWindowExceeded => StopReason::ContextExceeded,
            BedrockStop::GuardrailIntervened | BedrockStop::ContentFiltered => StopReason::Refused,
            _ => StopReason::Other,
        };

        let mut text = String::new();
        let mut reasoning = Vec::new();
        let mut tools_call = Vec::new();
        for block in message.content() {
            match block {
                ContentBlock::Text(t) => text.push_str(t),
                // opaque: the tool loop sends them back as they are
                ContentBlock::ReasoningContent(ReasoningContentBlock::ReasoningText(r)) => {
                    reasoning.push(json!({ "text": r.text(), "signature": r.signature() }))
                }
                ContentBlock::ReasoningContent(ReasoningContentBlock::RedactedContent(b)) => {
                    reasoning.push(json!({ "redacted": STANDARD.encode(b.as_ref()) }))
                }
                // a refusal asks for nothing: its tool calls are never run
                ContentBlock::ToolUse(_) if stop_reason == StopReason::Refused => {}
                ContentBlock::ToolUse(t) => {
                    // a cut answer may hold a tool call cut in the middle
                    if stop_reason.is_cut() {
                        return Err(self.who.truncated());
                    }
                    let arguments = from_document(t.input());
                    if !arguments.is_object() {
                        return Err(self.who.invalid("tool call arguments are not a JSON object"));
                    }
                    tools_call.push(ToolCall {
                        call_id: t.tool_use_id().to_string(),
                        function_name: t.name().to_string(),
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

        let (mut message, mut structured) = (None, None);
        // a refusal is a text, schema or not: the message of the guardrail
        if schema && stop_reason != StopReason::Refused {
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

        // the cached tokens are counted apart from inputTokens
        let u = out.usage();
        let usage = u
            .map(|u| Usage {
                input_tokens: u.input_tokens().max(0) as u64,
                cache_read_tokens: u.cache_read_input_tokens().unwrap_or(0).max(0) as u64,
                cache_write_tokens: u.cache_write_input_tokens().unwrap_or(0).max(0) as u64,
                output_tokens: u.output_tokens().max(0) as u64,
                reasoning_tokens: 0,
            })
            .unwrap_or_default();

        Ok(ModelResponse {
            message,
            structured,
            media: Vec::new(),
            tools_call,
            reasoning,
            stop_reason,
            stop_detail: Some(stop.as_str().to_string()),
            info: ResponseInfo {
                provider: self.who.provider.into(),
                model: self.model.clone(),
                execution_time,
                usage,
            },
        })
    }

    /// 401/403-like errors are about the credentials; the other service errors are the provider's.
    pub(super) fn sdk_error<E, R>(&self, e: SdkError<E, R>) -> ModelError
    where
        E: ProvideErrorMetadata + std::error::Error + Send + Sync + 'static,
        R: std::fmt::Debug + Send + Sync + 'static,
    {
        match &e {
            SdkError::ServiceError(s) => {
                let meta = s.err();
                // an error the SDK could not classify has no code: its whole chain says more
                let Some(code) = meta.code() else {
                    return self
                        .who
                        .provider_error(None, aws_sdk_bedrockruntime::error::DisplayErrorContext(&e).to_string());
                };
                let message = meta.message().map(str::to_string).unwrap_or_else(|| meta.to_string());
                match code {
                    "AccessDeniedException"
                    | "UnrecognizedClientException"
                    | "ExpiredTokenException"
                    | "InvalidSignatureException"
                    | "IncompleteSignatureException" => self.who.credentials(format!("{code}: {message}")),
                    _ => self.who.provider_error(None, format!("{code}: {message}")),
                }
            }
            other => {
                let text = aws_sdk_bedrockruntime::error::DisplayErrorContext(other).to_string();
                if text.to_lowercase().contains("credentials") {
                    self.who.credentials(text)
                } else {
                    self.who.network_text(text)
                }
            }
        }
    }
}
