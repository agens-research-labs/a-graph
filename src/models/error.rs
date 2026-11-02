//! What a call to a model can fail with. Every variant says which model and which provider;
//! the caller adds the node. `refused`, `context_exceeded` and `max_tokens` without a schema
//! are NOT errors: they are a `StopReason`.

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

    #[error("model `{model}` ({provider}): credentials: {detail}")]
    Credentials {
        model: String,
        provider: String,
        detail: String,
    },

    #[error("model `{model}` ({provider}): network: {source}")]
    Network {
        model: String,
        provider: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// The provider answered with an error, or a batch failed or expired.
    #[error("model `{model}` ({provider}): the provider answered {}: {message}", status.map_or("with an error".to_string(), |s| s.to_string()))]
    Provider {
        model: String,
        provider: String,
        status: Option<u16>,
        message: String,
    },

    #[error("model `{model}` ({provider}): invalid response: {detail}")]
    InvalidResponse {
        model: String,
        provider: String,
        detail: String,
    },

    /// The output reached `max_tokens` while a schema was required: the JSON is cut.
    #[error("model `{model}` ({provider}): the output was truncated at max_tokens")]
    Truncated { model: String, provider: String },

    #[error("model `{model}` ({provider}): expected media `{expected}`, produced `{produced}`")]
    MimeMismatch {
        model: String,
        provider: String,
        expected: String,
        produced: String,
    },
}
