//! Which model and provider an error is about: every provider builds its `ModelError`s here.

use super::error::ModelError;

#[derive(Debug, Clone)]
pub(crate) struct Identity {
    pub(crate) model: String,
    pub(crate) provider: &'static str,
}

impl Identity {
    pub(crate) fn new(model: &str, provider: &'static str) -> Self {
        Identity {
            model: model.to_string(),
            provider,
        }
    }

    fn parts(&self) -> (String, String) {
        (self.model.clone(), self.provider.to_string())
    }

    pub(crate) fn unsupported(&self, problems: Vec<String>) -> ModelError {
        let (model, provider) = self.parts();
        ModelError::Unsupported {
            model,
            provider,
            problems,
        }
    }

    pub(crate) fn invalid(&self, detail: impl Into<String>) -> ModelError {
        let (model, provider) = self.parts();
        ModelError::InvalidResponse {
            model,
            provider,
            detail: detail.into(),
        }
    }

    pub(crate) fn credentials(&self, detail: impl Into<String>) -> ModelError {
        let (model, provider) = self.parts();
        ModelError::Credentials {
            model,
            provider,
            detail: detail.into(),
        }
    }

    pub(crate) fn provider_error(&self, status: Option<u16>, message: impl Into<String>) -> ModelError {
        let (model, provider) = self.parts();
        ModelError::Provider {
            model,
            provider,
            status,
            message: message.into(),
        }
    }

    pub(crate) fn network(&self, e: reqwest::Error) -> ModelError {
        let (model, provider) = self.parts();
        ModelError::Network {
            model,
            provider,
            source: Box::new(e),
        }
    }

    /// A failure to reach the server that is not a `reqwest` error (the AWS SDK has its own).
    pub(crate) fn network_text(&self, detail: String) -> ModelError {
        let (model, provider) = self.parts();
        ModelError::Network {
            model,
            provider,
            source: detail.into(),
        }
    }

    pub(crate) fn truncated(&self) -> ModelError {
        let (model, provider) = self.parts();
        ModelError::Truncated { model, provider }
    }

    pub(crate) fn mime_mismatch(&self, expected: &str, produced: &str) -> ModelError {
        let (model, provider) = self.parts();
        ModelError::MimeMismatch {
            model,
            provider,
            expected: expected.into(),
            produced: produced.into(),
        }
    }
}
