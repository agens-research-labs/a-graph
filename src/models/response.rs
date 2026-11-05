//! The output of a call.

use std::ops::{Add, AddAssign};

use serde_json::Value;

use super::invocation::ToolCall;

#[derive(Debug, Clone)]
pub struct ModelResponse {
    /// The text, when the node has no schema or the model refused.
    pub message: Option<String>,
    /// The JSON, when the node has a schema.
    pub structured: Option<Value>,
    /// Zero or more produced media.
    pub media: Vec<MediaOutput>,
    /// Goes to the gate. Empty unless `stop_reason` is `ToolCall`.
    pub tools_call: Vec<ToolCall>,
    /// The original provider blocks, opaque: the tool loop sends them back.
    pub reasoning: Vec<Value>,
    pub stop_reason: StopReason,
    /// The provider's original value of the stop reason.
    pub stop_detail: Option<String>,
    pub info: ResponseInfo,
}

#[derive(Debug, Clone)]
pub struct MediaOutput {
    pub mime_type: String,
    /// Base64.
    pub data: String,
    /// The id of the media resource it is, put by the node that made it; a provider leaves it empty.
    pub id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    End,
    ToolCall,
    MaxTokens,
    ContextExceeded,
    Refused,
    Other,
}

#[derive(Debug, Clone)]
pub struct ResponseInfo {
    pub provider: String,
    pub model: String,
    /// Seconds, from the request sent to the full response received, network included.
    pub execution_time: f64,
    pub usage: Usage,
}

/// The common numeric counts; each trait converts its own. The tool loop sums them field by field.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Usage {
    /// Not from cache.
    pub input_tokens: u64,
    pub cache_read_tokens: u64,
    /// 0 where the provider does not count it.
    pub cache_write_tokens: u64,
    /// Everything generated, reasoning included.
    pub output_tokens: u64,
    /// Informational: already inside `output_tokens`.
    pub reasoning_tokens: u64,
}

impl Add for Usage {
    type Output = Usage;
    fn add(self, o: Usage) -> Usage {
        Usage {
            input_tokens: self.input_tokens + o.input_tokens,
            cache_read_tokens: self.cache_read_tokens + o.cache_read_tokens,
            cache_write_tokens: self.cache_write_tokens + o.cache_write_tokens,
            output_tokens: self.output_tokens + o.output_tokens,
            reasoning_tokens: self.reasoning_tokens + o.reasoning_tokens,
        }
    }
}

impl AddAssign for Usage {
    fn add_assign(&mut self, o: Usage) {
        *self = *self + o;
    }
}
