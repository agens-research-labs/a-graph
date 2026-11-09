//! Ollama (and any server that speaks its contract) through `POST /api/chat`, over REST.
//!
//! The `custom` provider of the yml is the same code with another name in the errors: a model
//! server of your own must speak the Ollama contract. No model table: what a model does not
//! accept (tools, thinking...) comes back as the server's own error, shown as it is. The server
//! is local: no key, and nothing to switch off by hand.

mod request;
mod response;

use async_trait::async_trait;
use serde_json::Value;

use super::http;
use super::identity::Identity;
use super::spec::LocalModel;
use super::timing::timed;
use super::{
    BatchEvents, BatchRequest, BatchReturn, Llm, ModelError, ModelInvocation, ModelRequirements, ModelResponse,
};
use crate::error::{Error, Path, Phase, Rule};
use crate::resources::MediaKind;

pub struct Ollama {
    who: Identity,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl Ollama {
    /// A model of provider `ollama`; `at` is where it is in the yml.
    // named as the other constructors of models are, after the provider
    #[allow(clippy::self_named_constructors)]
    pub fn ollama(m: &LocalModel, at: &Path) -> Result<Self, Error> {
        Self::assemble(m, "ollama", at)
    }

    /// A model of provider `custom`: a server that speaks the Ollama contract.
    pub fn custom(m: &LocalModel, at: &Path) -> Result<Self, Error> {
        Self::assemble(m, "custom", at)
    }

    fn assemble(m: &LocalModel, provider: &'static str, at: &Path) -> Result<Self, Error> {
        let client = http::client().map_err(|e| {
            Error::new(
                Phase::Interpretation,
                at.clone(),
                Rule::Io,
                format!("cannot build the HTTP client: {e}"),
            )
        })?;
        Ok(Ollama {
            who: Identity::new(&m.common.id, provider),
            model: m.common.model.clone(),
            base_url: m.host.trim_end_matches('/').to_string(),
            client,
        })
    }

    /// The server answers `{"error": "..."}`.
    fn api_error(&self, status: u16, body: &str) -> ModelError {
        let said = serde_json::from_str::<Value>(body)
            .ok()
            .and_then(|v| v.get("error").and_then(Value::as_str).map(str::to_string))
            .unwrap_or_else(|| body.chars().take(500).collect());
        self.who.provider_said(Some(status), None, "", Some(said))
    }
}

#[async_trait]
impl Llm for Ollama {
    fn check(&self, req: &ModelRequirements) -> Result<(), ModelError> {
        let mut problems = Vec::new();
        if !req.output_media.is_empty() {
            problems.push("this server produces text only".to_string());
        }
        if req.batch {
            problems.push("this server has no batch".to_string());
        }
        for t in &req.input_media {
            match t {
                MediaKind::Md | MediaKind::Img(_) => {}
                MediaKind::File => problems.push(
                    "a file of any format cannot go to a model: a utility has to change it into text first".to_string(),
                ),
                MediaKind::Audio(_) | MediaKind::Video(_) => {
                    problems.push("audio and video input are not supported yet (text and images only)".to_string())
                }
            }
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(self.who.unsupported(problems))
        }
    }

    async fn invoke(&self, call: &ModelInvocation) -> Result<ModelResponse, ModelError> {
        let body = self.request_body(call)?;
        let req = self.client.post(format!("{}/api/chat", self.base_url)).json(&body);
        let (text, secs) = timed(http::send(&self.who, req, |status, body| self.api_error(status, body))).await;
        let value: Value =
            serde_json::from_str(&text?).map_err(|e| self.who.invalid(format!("the answer is not JSON: {e}")))?;
        self.parse_chat(call.schema.is_some(), &value, secs)
    }

    async fn invoke_batch(&self, _requests: Vec<BatchRequest>, _events: BatchEvents) -> BatchReturn {
        Err(self.who.unsupported(vec!["this server has no batch".into()]))
    }
}
