//! From a `ModelInvocation` to a `Converse` call.

use aws_sdk_bedrockruntime::Client;
use aws_sdk_bedrockruntime::operation::converse::ConverseOutput;
use aws_sdk_bedrockruntime::types::{
    ContentBlock, ConversationRole, ImageBlock, ImageSource, InferenceConfiguration, JsonSchemaDefinition, Message,
    OutputConfig, OutputFormat, OutputFormatStructure, OutputFormatType, ReasoningContentBlock, ReasoningTextBlock,
    SystemContentBlock, Tool, ToolConfiguration, ToolInputSchema, ToolResultBlock, ToolResultContentBlock,
    ToolResultStatus, ToolSpecification, ToolUseBlock,
};
use aws_smithy_types::Blob;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};

use super::Bedrock;
use super::convert::{image_format, to_document};
use crate::models::Reasoning;
use crate::models::{ChatEntry, MediaInput, ModelError, ModelInvocation, Role, ToolRound};

impl Bedrock {
    pub(super) async fn converse(&self, client: &Client, call: &ModelInvocation) -> Result<ConverseOutput, ModelError> {
        let mut req = client.converse().model_id(&self.model);

        if !call.system.is_empty() {
            req = req.system(SystemContentBlock::Text(call.system.clone()));
        }
        for m in self.messages(call)? {
            req = req.messages(m);
        }

        if call.max_tokens.is_some() || call.temperature.is_some() || call.top_p.is_some() {
            req = req.inference_config(
                InferenceConfiguration::builder()
                    .set_max_tokens(call.max_tokens.map(|n| n as i32))
                    .set_temperature(call.temperature.map(|t| t as f32))
                    .set_top_p(call.top_p.map(|p| p as f32))
                    .build(),
            );
        }

        if !call.tools.is_empty() {
            let mut cfg = ToolConfiguration::builder();
            for t in &call.tools {
                let spec = ToolSpecification::builder()
                    .name(&t.name)
                    .description(&t.description)
                    .input_schema(ToolInputSchema::Json(to_document(&t.parameters)))
                    .build()
                    .map_err(|e| self.who.unsupported(vec![format!("tool `{}`: {e}", t.name)]))?;
                cfg = cfg.tools(Tool::ToolSpec(spec));
            }
            req = req.tool_config(cfg.build().map_err(|e| self.who.unsupported(vec![e.to_string()]))?);
        }

        // structured output and the reasoning effort live in the output config
        let effort = match call.reasoning {
            None | Some(Reasoning::Off) => None,
            Some(Reasoning::Minimal | Reasoning::Low) => Some("low"),
            Some(Reasoning::Medium) => Some("medium"),
            Some(Reasoning::High) => Some("high"),
            Some(Reasoning::Max) => Some("max"),
        };
        if effort.is_some() || call.schema.is_some() {
            let mut oc = OutputConfig::builder();
            if let Some(e) = effort {
                oc = oc.effort(e);
            }
            if let Some(schema) = &call.schema {
                // Converse takes the schema as a JSON string
                let def = JsonSchemaDefinition::builder()
                    .schema(schema.to_string())
                    .name("output")
                    .build()
                    .map_err(|e| self.who.unsupported(vec![e.to_string()]))?;
                let format = OutputFormat::builder()
                    .r#type(OutputFormatType::JsonSchema)
                    .structure(OutputFormatStructure::JsonSchema(def))
                    .build()
                    .map_err(|e| self.who.unsupported(vec![e.to_string()]))?;
                oc = oc.text_format(format);
            }
            req = req.output_config(oc.build());
        }

        // the thinking switch is a field of the model itself (the way Claude models take it)
        match call.reasoning {
            None => {}
            Some(Reasoning::Off) => {
                req = req.additional_model_request_fields(to_document(&json!({ "thinking": { "type": "disabled" } })))
            }
            Some(_) => {
                req = req.additional_model_request_fields(to_document(
                    &json!({ "thinking": { "type": "adaptive", "display": "summarized" } }),
                ))
            }
        }

        req.send().await.map_err(|e| self.sdk_error(e))
    }

    fn messages(&self, call: &ModelInvocation) -> Result<Vec<Message>, ModelError> {
        let mut messages = Vec::new();
        let build = |role: ConversationRole, content: Vec<ContentBlock>| {
            Message::builder().role(role).set_content(Some(content)).build()
        };
        let fail = |e: aws_smithy_types::error::operation::BuildError| self.who.unsupported(vec![e.to_string()]);

        let entry = |e: &ChatEntry| {
            let role = match e.role {
                Role::User => ConversationRole::User,
                Role::Assistant => ConversationRole::Assistant,
            };
            build(role, vec![ContentBlock::Text(e.content.clone())]).map_err(fail)
        };
        for e in &call.history {
            messages.push(entry(e)?);
        }

        let mut content = Vec::new();
        for m in &call.media {
            match m {
                MediaInput::Markdown(md) => content.push(ContentBlock::Text(md.clone())),
                MediaInput::Image { format, data } => {
                    let f = image_format(format).ok_or_else(|| {
                        self.who.unsupported(vec![format!(
                            "image format `{format}` is not accepted (png, jpeg, gif, webp)"
                        )])
                    })?;
                    let bytes = STANDARD.decode(data).map_err(|e| {
                        self.who
                            .unsupported(vec![format!("the image is not valid base64: {e}")])
                    })?;
                    let block = ImageBlock::builder()
                        .format(f)
                        .source(ImageSource::Bytes(Blob::new(bytes)))
                        .build()
                        .map_err(fail)?;
                    content.push(ContentBlock::Image(block));
                }
            }
        }
        if !call.user.is_empty() {
            content.push(ContentBlock::Text(call.user.clone()));
        }
        messages.push(build(ConversationRole::User, content).map_err(fail)?);

        for round in &call.tool_rounds {
            messages.extend(self.tool_round(round)?);
        }
        // the corrections of `schema_prompt` come last, after the tool rounds
        for e in &call.corrections {
            messages.push(entry(e)?);
        }
        Ok(messages)
    }

    /// The assistant message that asked for the tools (the reasoning blocks first, as they came),
    /// then one user message with the results.
    fn tool_round(&self, round: &ToolRound) -> Result<Vec<Message>, ModelError> {
        let fail = |e: aws_smithy_types::error::operation::BuildError| self.who.unsupported(vec![e.to_string()]);
        let mut assistant = Vec::new();
        for r in &round.reasoning {
            assistant.push(ContentBlock::ReasoningContent(self.reasoning_block(r)?));
        }
        if let Some(m) = round.message.as_deref().filter(|m| !m.is_empty()) {
            assistant.push(ContentBlock::Text(m.to_string()));
        }
        for c in &round.calls {
            let block = ToolUseBlock::builder()
                .tool_use_id(&c.call_id)
                .name(&c.function_name)
                .input(to_document(&c.arguments))
                .build()
                .map_err(fail)?;
            assistant.push(ContentBlock::ToolUse(block));
        }
        let mut results = Vec::new();
        for r in &round.results {
            let text = match &r.result {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            let block = ToolResultBlock::builder()
                .tool_use_id(&r.call_id)
                .content(ToolResultContentBlock::Text(text))
                .status(if r.is_error {
                    ToolResultStatus::Error
                } else {
                    ToolResultStatus::Success
                })
                .build()
                .map_err(fail)?;
            results.push(ContentBlock::ToolResult(block));
        }
        Ok(vec![
            Message::builder()
                .role(ConversationRole::Assistant)
                .set_content(Some(assistant))
                .build()
                .map_err(fail)?,
            Message::builder()
                .role(ConversationRole::User)
                .set_content(Some(results))
                .build()
                .map_err(fail)?,
        ])
    }

    /// A reasoning block as the response stored it: `{"text", "signature"}` or `{"redacted"}`.
    fn reasoning_block(&self, v: &Value) -> Result<ReasoningContentBlock, ModelError> {
        if let Some(text) = v.get("text").and_then(Value::as_str) {
            let block = ReasoningTextBlock::builder()
                .text(text)
                .set_signature(v.get("signature").and_then(Value::as_str).map(str::to_string))
                .build()
                .map_err(|e| self.who.unsupported(vec![e.to_string()]))?;
            return Ok(ReasoningContentBlock::ReasoningText(block));
        }
        if let Some(b64) = v.get("redacted").and_then(Value::as_str) {
            let bytes = STANDARD.decode(b64).map_err(|e| {
                self.who
                    .invalid(format!("a stored redacted reasoning block is not base64: {e}"))
            })?;
            return Ok(ReasoningContentBlock::RedactedContent(Blob::new(bytes)));
        }
        Err(self
            .who
            .invalid("a stored reasoning block has neither `text` nor `redacted`"))
    }
}
