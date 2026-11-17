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
    /// The tokens of the answer with their log probabilities, one list for each call to the model, in order.
    /// A provider that gives them has one list for its call, empty when the answer has none; the tool loop
    /// and the controller of a `react` put the lists of all their calls together. Empty for a provider that
    /// never gives them.
    pub logprobs: Vec<Vec<TokenLogprob>>,
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

/// One token of an answer, as the model wrote it, and how likely it was.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenLogprob {
    pub token: String,
    /// The natural logarithm of the probability of the token.
    pub logprob: f64,
    /// The bytes of the token, when the provider gives them: a token may hold part of a character.
    pub bytes: Option<Vec<u8>>,
    /// The most likely tokens at the same place, as many as were asked, from the provider.
    pub top_logprobs: Vec<TopLogprob>,
}

/// A token the model could have written instead.
#[derive(Debug, Clone, PartialEq)]
pub struct TopLogprob {
    pub token: String,
    pub logprob: f64,
    pub bytes: Option<Vec<u8>>,
}

/// The tokens of a list of the providers that give them (OpenAI, Mistral, Ollama): each one
/// `{ token, logprob, bytes, top_logprobs: [ { token, logprob, bytes } ] }`. A token without `token` or
/// `logprob` is an error, given back as the text of the problem.
pub(crate) fn token_logprobs(list: &[Value]) -> Result<Vec<TokenLogprob>, String> {
    list.iter()
        .map(|t| {
            let (token, logprob, bytes) = token_parts(t)?;
            let top_logprobs = t
                .get("top_logprobs")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(|a| token_parts(a).map(|(token, logprob, bytes)| TopLogprob { token, logprob, bytes }))
                .collect::<Result<_, _>>()?;
            Ok(TokenLogprob {
                token,
                logprob,
                bytes,
                top_logprobs,
            })
        })
        .collect()
}

fn token_parts(t: &Value) -> Result<(String, f64, Option<Vec<u8>>), String> {
    let token = t
        .get("token")
        .and_then(Value::as_str)
        .ok_or("a logprob without `token`")?
        .to_string();
    let logprob = t
        .get("logprob")
        .and_then(Value::as_f64)
        .ok_or("a logprob without `logprob`")?;
    let bytes = match t.get("bytes") {
        None | Some(Value::Null) => None,
        Some(Value::Array(b)) => Some(
            b.iter()
                .map(|n| n.as_u64().and_then(|n| u8::try_from(n).ok()))
                .collect::<Option<Vec<u8>>>()
                .ok_or("a logprob whose `bytes` are not bytes")?,
        ),
        Some(_) => return Err("a logprob whose `bytes` are not a list".into()),
    };
    Ok((token, logprob, bytes))
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

impl StopReason {
    /// The answer stopped before its end: at `max_tokens` or at the end of the context window. A tool call in
    /// it may be cut anywhere, so it is never run.
    pub fn is_cut(self) -> bool {
        matches!(self, StopReason::MaxTokens | StopReason::ContextExceeded)
    }
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
