//! Mistral's own API (`api.mistral.ai`), chat completions, called directly over REST.
//!
//! Mistral hosted on Bedrock, Vertex AI or Azure is not this provider. A produced speech goes to
//! `/v1/audio/speech`. No model table: what a model does not accept comes back as Mistral's own
//! error, shown as it is.

mod batch;
mod request;
mod response;
mod speech;

use async_trait::async_trait;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde_json::Value;
use std::fmt;
use std::time::Duration;

use super::http::{self, ErrorShape, refused_credentials};
use super::identity::Identity;
use super::spec::ApiModel;
use super::timing::timed;
use super::{
    BatchEvents, BatchRequest, BatchReturn, Llm, ModelError, ModelInvocation, ModelRequirements, ModelResponse,
};
use crate::error::{Error, Path, Phase, Rule};
use crate::resources::MediaKind;
use crate::schema::key;

pub(super) const PROVIDER: &str = "mistral";
const BASE_URL: &str = "https://api.mistral.ai";
/// How often a batch is polled. There is no maximum wait: it ends when Mistral says so.
const POLL_INTERVAL: Duration = Duration::from_secs(30);

/// `{ "message" }`, or `{ "detail": [...] }` for a body it cannot read. No fixed code: a validation error
/// lists what was sent (`detail[].input`).
const ERRORS: ErrorShape = ErrorShape {
    code: &[],
    message: &["/message", "/detail"],
    credentials: refused_credentials,
};

pub struct Mistral {
    who: Identity,
    model: String,
    api_key: String,
    base_url: String,
    poll_interval: Duration,
    client: reqwest::Client,
}

impl fmt::Debug for Mistral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mistral")
            .field("model", &self.model)
            .field("api_key", &"<redacted>")
            .finish()
    }
}

impl Mistral {
    /// Reads the key from the environment variable the yml names; `at` is where the model is in
    /// the yml. A variable that is not set is an error now, not at the first call.
    pub fn new(m: &ApiModel, at: &Path) -> Result<Self, Error> {
        let api_key = std::env::var(&m.api_key).map_err(|_| {
            Error::new(
                Phase::Interpretation,
                at.key(key::API_KEY),
                Rule::EnvMissing,
                format!("environment variable `{}` is not set", m.api_key),
            )
        })?;
        let client = http::client().map_err(|e| {
            Error::new(
                Phase::Interpretation,
                at.clone(),
                Rule::Io,
                format!("cannot build the HTTP client: {e}"),
            )
        })?;
        Ok(Mistral {
            who: Identity::new(&m.common.id, PROVIDER),
            model: m.common.model.clone(),
            api_key,
            base_url: BASE_URL.into(),
            poll_interval: POLL_INTERVAL,
            client,
        })
    }

    /// For tests: another server. Not reachable from the yml, whose host the allow-list checks.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into().trim_end_matches('/').to_string();
        self
    }

    pub fn with_poll_interval(mut self, every: Duration) -> Self {
        self.poll_interval = every;
        self
    }

    fn headers(&self) -> Result<HeaderMap, ModelError> {
        let mut v = HeaderValue::from_str(&format!("Bearer {}", self.api_key)).map_err(|_| {
            self.who
                .credentials("the API key has characters not allowed in a header")
        })?;
        v.set_sensitive(true);
        let mut h = HeaderMap::new();
        h.insert(AUTHORIZATION, v);
        Ok(h)
    }

    async fn post_text(&self, path: &str, body: &Value) -> Result<String, ModelError> {
        let req = self
            .client
            .post(format!("{}{path}", self.base_url))
            .headers(self.headers()?)
            .json(body);
        http::send(&self.who, req, &ERRORS).await
    }

    async fn get_text(&self, path: &str) -> Result<String, ModelError> {
        let req = self
            .client
            .get(format!("{}{path}", self.base_url))
            .headers(self.headers()?);
        http::send(&self.who, req, &ERRORS).await
    }
}

#[async_trait]
impl Llm for Mistral {
    fn check(&self, req: &ModelRequirements) -> Result<(), ModelError> {
        let mut problems = Vec::new();
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
        if let Err(mut p) = self.speech_format(&req.output_media) {
            problems.append(&mut p);
        }
        if req.logprobs.is_some() && !req.output_media.is_empty() {
            problems.push("a produced speech gives no logprobs".to_string());
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(self.who.unsupported(problems))
        }
    }

    async fn invoke(&self, call: &ModelInvocation) -> Result<ModelResponse, ModelError> {
        if let Some(format) = self
            .speech_format(&call.output_media)
            .map_err(|p| self.who.unsupported(p))?
        {
            return self.invoke_speech(call, &format).await;
        }
        let body = self.request_body(call)?;
        let (text, secs) = timed(self.post_text("/v1/chat/completions", &body)).await;
        let value: Value =
            serde_json::from_str(&text?).map_err(|e| self.who.invalid(format!("the answer is not JSON: {e}")))?;
        self.parse_completion(&value, secs)
    }

    async fn invoke_batch(&self, requests: Vec<BatchRequest>, events: BatchEvents) -> BatchReturn {
        self.run_batch(requests, events).await
    }

    async fn cancel_batch(&self, batch_id: &str) -> Result<(), ModelError> {
        self.cancel(batch_id).await
    }
}
