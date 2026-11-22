//! What a call to a model can fail with. Every variant says which model and which provider;
//! the caller adds the node. `refused`, `context_exceeded` and `max_tokens` without a schema
//! are NOT errors: they are a `StopReason`; a cut answer that asks for a tool is one (`Truncated`).

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelError {
    /// The provider cannot do what the node asks (found by `check`, before any call).
    #[error("model `{model}` ({provider}) does not support what the node asks: {}", problems.join("; "))]
    Unsupported {
        model: String,
        provider: String,
        problems: Vec<String>,
    },

    /// `said` is what the provider wrote, if it did: a content, never part of the message.
    #[error("model `{model}` ({provider}): credentials: {detail}")]
    Credentials {
        model: String,
        provider: String,
        detail: String,
        said: Option<String>,
    },

    #[error("model `{model}` ({provider}): network: {source}")]
    Network {
        model: String,
        provider: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// The provider answered with an error, or a batch failed or expired. `code` is the fixed code the
    /// provider gave the error, if it is one (see `fixed_code`); `said` is what the provider wrote: a
    /// content, never part of the message.
    #[error("model `{model}` ({provider}): {}", answered(*status, code.as_deref(), message))]
    Provider {
        model: String,
        provider: String,
        status: Option<u16>,
        code: Option<String>,
        message: String,
        said: Option<String>,
    },

    /// `said` is a value the provider chose that `fixed_code` does not keep: a content, never part
    /// of the message.
    #[error("model `{model}` ({provider}): invalid response: {detail}")]
    InvalidResponse {
        model: String,
        provider: String,
        detail: String,
        said: Option<String>,
    },

    /// The output stopped before its end (`StopReason::is_cut`) while a schema was required, so the JSON is cut;
    /// or while it asked for a tool, so the call may be cut and is not run. `cut` says where it stopped.
    #[error("model `{model}` ({provider}): the output was cut before its end{}", cut_text(*cut))]
    Truncated { model: String, provider: String, cut: Cut },

    #[error("model `{model}` ({provider}): expected media `{expected}`, produced `{produced}`")]
    MimeMismatch {
        model: String,
        provider: String,
        expected: String,
        produced: String,
    },
}

/// Where a cut output stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cut {
    /// At `max_tokens`.
    MaxTokens,
    /// At the end of the context window of the model.
    ContextWindow,
}

/// What a cut output adds to its message: where it stopped, and what to do about it.
fn cut_text(cut: Cut) -> &'static str {
    match cut {
        Cut::MaxTokens => ", at `max_tokens`: increase `max_tokens`",
        Cut::ContextWindow => ": the request exceeded the context window of the model",
    }
}

impl ModelError {
    /// What the provider wrote that made the error, if it did: a content, never part of the message.
    pub fn said(&self) -> Option<&str> {
        match self {
            ModelError::Credentials { said, .. }
            | ModelError::Provider { said, .. }
            | ModelError::InvalidResponse { said, .. } => said.as_deref(),
            _ => None,
        }
    }
}

/// `the provider answered 400 (invalid_request_error): <message>`, each part only when there is one.
pub(crate) fn answered(status: Option<u16>, code: Option<&str>, message: &str) -> String {
    let mut text = format!(
        "the provider answered {}",
        status.map_or("with an error".to_string(), |s| s.to_string())
    );
    if let Some(code) = code {
        text.push_str(&format!(" ({code})"));
    }
    if !message.is_empty() {
        text.push_str(&format!(": {message}"));
    }
    text
}

/// `code` if it is a fixed code (letters, digits and `_`, 64 characters at most), which says what kind
/// of error it is and holds no content; `None` otherwise.
pub(crate) fn fixed_code(code: Option<&str>) -> Option<String> {
    code.filter(|c| !c.is_empty() && c.len() <= 64 && c.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_'))
        .map(str::to_string)
}
