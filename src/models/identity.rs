//! Which model and provider an error is about: every provider builds its `ModelError`s here.

use super::error::{ModelError, answered, fixed_code};

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
            said: None,
        }
    }

    /// Like `invalid`, but `said` is a value the provider chose that `fixed_code` does not keep.
    pub(crate) fn invalid_said(&self, detail: impl Into<String>, said: impl Into<String>) -> ModelError {
        let (model, provider) = self.parts();
        ModelError::InvalidResponse {
            model,
            provider,
            detail: detail.into(),
            said: Some(said.into()),
        }
    }

    /// Credentials that are not valid, as `detail` says; `detail` holds nothing the provider wrote.
    pub(crate) fn credentials(&self, detail: impl Into<String>) -> ModelError {
        let (model, provider) = self.parts();
        ModelError::Credentials {
            model,
            provider,
            detail: detail.into(),
            said: None,
        }
    }

    /// The provider refused the credentials (401, 403) with the code `code` and the text `said`.
    pub(crate) fn credentials_refused(
        &self,
        status: Option<u16>,
        code: Option<&str>,
        said: Option<String>,
    ) -> ModelError {
        let (model, provider) = self.parts();
        ModelError::Credentials {
            model,
            provider,
            detail: answered(status, fixed_code(code).as_deref(), ""),
            said,
        }
    }

    /// An error of the provider that `message` says; `message` holds nothing the provider wrote.
    pub(crate) fn provider_error(&self, status: Option<u16>, message: impl Into<String>) -> ModelError {
        let (model, provider) = self.parts();
        ModelError::Provider {
            model,
            provider,
            status,
            code: None,
            message: message.into(),
            said: None,
        }
    }

    /// An error the provider answered with the code `code` and the text `said`. `message` says where
    /// (`batch request failed`), or is empty.
    pub(crate) fn provider_said(
        &self,
        status: Option<u16>,
        code: Option<&str>,
        message: impl Into<String>,
        said: Option<String>,
    ) -> ModelError {
        let (model, provider) = self.parts();
        ModelError::Provider {
            model,
            provider,
            status,
            code: fixed_code(code),
            message: message.into(),
            said,
        }
    }

    pub(crate) fn network(&self, e: reqwest::Error) -> ModelError {
        let (model, provider) = self.parts();
        ModelError::Network {
            model,
            provider,
            // Destination metadata is recorded separately; URLs can contain credentials or query secrets.
            source: Box::new(e.without_url()),
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
