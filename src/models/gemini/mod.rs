//! Gemini through the Interactions API (`POST /v1beta/interactions`), called directly over REST,
//! from the Gemini API (an API key) or from Vertex AI (publisher google, ambient credentials).
//!
//! Stateless (`store: false`): every call sends the whole conversation. As for Anthropic there is
//! no model table: what a model does not accept comes back as Google's own error, shown as it is.
//! Batch is not available in the Interactions API yet: `check` says so.

mod media;
mod request;
mod response;

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;
use std::fmt;

use super::auth::{GoogleAdc, TokenSource};
use super::http::{self, ErrorShape, refused_credentials};
use super::identity::Identity;
use super::spec::{ApiModel, Model, VertexModel};
use super::timing::timed;
use super::{
    BatchEvents, BatchRequest, BatchReturn, Llm, ModelError, ModelInvocation, ModelRequirements, ModelResponse,
    Reasoning,
};
use crate::error::{Error, Path, Phase, Rule};
use crate::resources::MediaKind;
use crate::schema::key;

const INTERACTIONS_URL: &str = "https://generativelanguage.googleapis.com/v1beta/interactions";

enum Auth {
    /// `x-goog-api-key` (the Gemini API).
    Key(String),
    /// `Authorization: Bearer <token>`; the token comes from ambient credentials (Vertex AI).
    Bearer(Box<dyn TokenSource>),
}

/// `{ "error": { "code": 404, "message", "status": "NOT_FOUND" } }`, or the same inside a one-element array:
/// `code` is the HTTP status, the code of the error is `status`. A key that is not valid comes back as 400.
const ERRORS: ErrorShape = ErrorShape {
    code: &["/error/status"],
    message: &["/error/message"],
    credentials: |status, body| refused_credentials(status, body) || body.contains("API_KEY_INVALID"),
};

pub struct Gemini {
    who: Identity,
    model: String,
    auth: Auth,
    /// The whole URL of the call.
    url: String,
    client: reqwest::Client,
}

impl fmt::Debug for Gemini {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Gemini")
            .field("provider", &self.who.provider)
            .field("model", &self.model)
            .field("credentials", &"<redacted>")
            .finish()
    }
}

impl Gemini {
    /// The Gemini API. Reads the key from the environment variable the yml names; `at` is where
    /// the model is in the yml. A variable that is not set is an error now, not at the first call.
    pub fn new(m: &ApiModel, at: &Path) -> Result<Self, Error> {
        let api_key = std::env::var(&m.api_key).map_err(|_| {
            Error::new(
                Phase::Interpretation,
                at.key(key::API_KEY),
                Rule::EnvMissing,
                format!("environment variable `{}` is not set", m.api_key),
            )
        })?;
        Self::assemble(
            &m.common.id,
            &m.common.model,
            "gemini",
            Auth::Key(api_key),
            INTERACTIONS_URL.into(),
            at,
        )
    }

    /// Gemini on Vertex AI (publisher google), with the Google Application Default Credentials.
    pub fn vertex(m: &VertexModel, at: &Path) -> Result<Self, Error> {
        let host = Model::VertexAi(m.clone())
            .host()
            .map_err(|e| Error::new(Phase::Interpretation, at.clone(), Rule::BadValue, e))?;
        let url = format!(
            "https://{host}/v1beta1/projects/{}/locations/{}/interactions",
            m.project, m.location
        );
        Self::assemble(
            &m.common.id,
            &m.common.model,
            "vertex_ai",
            Auth::Bearer(Box::new(GoogleAdc::new())),
            url,
            at,
        )
    }

    fn assemble(
        id: &str,
        model: &str,
        provider: &'static str,
        auth: Auth,
        url: String,
        at: &Path,
    ) -> Result<Self, Error> {
        let client = http::client().map_err(|e| {
            Error::new(
                Phase::Interpretation,
                at.clone(),
                Rule::Io,
                format!("cannot build the HTTP client: {e}"),
            )
        })?;
        Ok(Gemini {
            who: Identity::new(id, provider),
            model: model.to_string(),
            auth,
            url,
            client,
        })
    }

    /// For tests: another server for the Gemini API. Not reachable from the yml, whose host the
    /// allow-list checks.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.url = format!("{}/v1beta/interactions", url.into().trim_end_matches('/'));
        self
    }

    /// For tests: another URL for the call of Vertex AI.
    pub fn with_interactions_url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }

    /// Another source of the bearer token (tests, or a token the caller already has).
    pub fn with_token_source(mut self, source: Box<dyn TokenSource>) -> Self {
        self.auth = Auth::Bearer(source);
        self
    }

    /// The URL of the call: the host the allow-list checked.
    pub fn endpoint(&self) -> &str {
        &self.url
    }

    async fn headers(&self) -> Result<HeaderMap, ModelError> {
        let bad = |what: &str| {
            self.who
                .credentials(format!("the {what} has characters not allowed in a header"))
        };
        let mut h = HeaderMap::new();
        match &self.auth {
            Auth::Key(k) => {
                let mut v = HeaderValue::from_str(k).map_err(|_| bad("API key"))?;
                v.set_sensitive(true);
                h.insert(HeaderName::from_static("x-goog-api-key"), v);
            }
            Auth::Bearer(source) => {
                let token = source.token().await.map_err(|detail| self.who.credentials(detail))?;
                let mut v = HeaderValue::from_str(&format!("Bearer {token}")).map_err(|_| bad("token"))?;
                v.set_sensitive(true);
                h.insert(reqwest::header::AUTHORIZATION, v);
            }
        }
        Ok(h)
    }
}

#[async_trait]
impl Llm for Gemini {
    fn check(&self, req: &ModelRequirements) -> Result<(), ModelError> {
        let mut problems = Vec::new();
        if req.reasoning == Some(Reasoning::Off) {
            problems.push("Gemini cannot switch thinking off: its levels are minimal, low, medium, high".to_string());
        }
        if req.batch {
            problems.push("batch is not available in the Gemini Interactions API yet".to_string());
        }
        if req.logprobs.is_some() {
            problems.push("the Gemini Interactions API gives no logprobs".to_string());
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
        if let Err(mut p) = self.media_format(&req.output_media) {
            problems.append(&mut p);
        }
        if req.schema.is_some() && !req.output_media.is_empty() {
            problems.push("an output schema and a produced media cannot be asked together".to_string());
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(self.who.unsupported(problems))
        }
    }

    async fn invoke(&self, call: &ModelInvocation) -> Result<ModelResponse, ModelError> {
        let body = self.request_body(call)?;
        let req = self.client.post(&self.url).headers(self.headers().await?).json(&body);
        let (text, secs) = timed(http::send(&self.who, req, &ERRORS)).await;
        let value: Value =
            serde_json::from_str(&text?).map_err(|e| self.who.invalid(format!("the answer is not JSON: {e}")))?;
        self.parse_interaction(call, &value, secs)
    }

    async fn invoke_batch(&self, _requests: Vec<BatchRequest>, _events: BatchEvents) -> BatchReturn {
        Err(self
            .who
            .unsupported(vec!["batch is not available in the Gemini Interactions API yet".into()]))
    }
}
