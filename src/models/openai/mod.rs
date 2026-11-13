//! OpenAI through the Responses API (`POST /v1/responses`), called directly over REST.
//!
//! Text, images and tools go to Responses. A produced media goes to the API that makes it: an
//! image to `/v1/images/generations`, a speech to `/v1/audio/speech`. No model table: what a model
//! does not accept comes back as OpenAI's own error, shown as it is. Azure Foundry with publisher
//! openai speaks the same API (see `OpenAi::azure`).

mod batch;
mod media;
mod request;
mod response;

use std::fmt;

use async_trait::async_trait;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;

use super::auth::{AzureAmbient, TokenSource};
use super::http::{self, ErrorShape, refused_credentials};
use super::identity::Identity;
use super::spec::{ApiModel, AzureCredentials, AzureModel};
use super::timing::timed;
use super::{
    BatchEvents, BatchRequest, BatchReturn, Llm, ModelError, ModelInvocation, ModelRequirements, ModelResponse,
};
use crate::error::{Error, Path, Phase, Rule};
use crate::resources::MediaKind;
use crate::schema::key;

const BASE_URL: &str = "https://api.openai.com/v1";

/// How a call proves who it is.
enum Auth {
    /// `Authorization: Bearer <key>` (OpenAI).
    Bearer(String),
    /// `api-key: <key>` (Azure).
    ApiKey(String),
    /// `Authorization: Bearer <token>`; the token comes from ambient credentials (Azure).
    Token(Box<dyn TokenSource>),
}

/// `{ "error": { "message", "type", "code" } }`: the code, or else the type, is the code.
pub(super) const ERRORS: ErrorShape = ErrorShape {
    code: &["/error/code", "/error/type"],
    message: &["/error/message"],
    credentials: refused_credentials,
};

pub struct OpenAi {
    who: Identity,
    model: String,
    auth: Auth,
    base_url: String,
    /// Azure: the Global-Batch deployment a batch is sent to.
    batch_model: Option<String>,
    poll_interval: std::time::Duration,
    client: reqwest::Client,
}

impl fmt::Debug for OpenAi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenAi")
            .field("provider", &self.who.provider)
            .field("model", &self.model)
            .field("key", &"<redacted>")
            .finish()
    }
}

impl OpenAi {
    /// OpenAI itself. Reads the key from the environment variable the yml names; `at` is where
    /// the model is in the yml. A variable that is not set is an error now, not at the first call.
    pub fn new(m: &ApiModel, at: &Path) -> Result<Self, Error> {
        let key = env_key(&m.api_key, &at.key(key::API_KEY))?;
        Self::assemble(
            &m.common.id,
            &m.common.model,
            "openai",
            Auth::Bearer(key),
            BASE_URL.into(),
            None,
            at,
        )
    }

    /// A model hosted by Azure Foundry (publisher openai): `resource` builds the endpoint, the
    /// `model` of the yml is the name of the deployment, and a `batch` block names the
    /// Global-Batch deployment.
    pub fn azure(m: &AzureModel, at: &Path) -> Result<Self, Error> {
        let auth = match &m.credentials {
            AzureCredentials::ApiKey(var) => Auth::ApiKey(env_key(var, &at.key(key::CREDENTIALS).key(key::API_KEY))?),
            AzureCredentials::Ambient => Auth::Token(Box::new(AzureAmbient::new(
                "https://cognitiveservices.azure.com/.default",
            ))),
        };
        let base = format!("https://{}.openai.azure.com/openai/v1", m.resource.to_lowercase());
        Self::assemble(
            &m.common.id,
            &m.common.model,
            "azure_foundry",
            auth,
            base,
            m.batch.as_ref().map(|b| b.deployment.clone()),
            at,
        )
    }

    fn assemble(
        id: &str,
        model: &str,
        provider: &'static str,
        auth: Auth,
        base_url: String,
        batch_model: Option<String>,
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
        Ok(OpenAi {
            who: Identity::new(id, provider),
            model: model.to_string(),
            auth,
            base_url,
            batch_model,
            poll_interval: batch::POLL_INTERVAL,
            client,
        })
    }

    /// Azure: the name of the Global-Batch deployment (a test can set it without a `batch` block).
    pub fn with_batch_deployment(mut self, deployment: impl Into<String>) -> Self {
        self.batch_model = Some(deployment.into());
        self
    }

    pub fn with_poll_interval(mut self, every: std::time::Duration) -> Self {
        self.poll_interval = every;
        self
    }

    /// Another source of the bearer token (tests, or a token the caller already has).
    pub fn with_token_source(mut self, source: Box<dyn TokenSource>) -> Self {
        self.auth = Auth::Token(source);
        self
    }

    /// The base URL of the API: the host the allow-list checked.
    pub fn endpoint(&self) -> &str {
        &self.base_url
    }

    /// For tests: another server (the whole base, `/v1` included). Not reachable from the yml,
    /// whose host the allow-list checks.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into().trim_end_matches('/').to_string();
        self
    }

    async fn headers(&self) -> Result<HeaderMap, ModelError> {
        let bad = || self.who.credentials("the key has characters not allowed in a header");
        let mut h = HeaderMap::new();
        match &self.auth {
            Auth::Bearer(k) => {
                let mut v = HeaderValue::from_str(&format!("Bearer {k}")).map_err(|_| bad())?;
                v.set_sensitive(true);
                h.insert(AUTHORIZATION, v);
            }
            Auth::ApiKey(k) => {
                let mut v = HeaderValue::from_str(k).map_err(|_| bad())?;
                v.set_sensitive(true);
                h.insert(HeaderName::from_static("api-key"), v);
            }
            Auth::Token(source) => {
                let token = source.token().await.map_err(|detail| self.who.credentials(detail))?;
                let mut v = HeaderValue::from_str(&format!("Bearer {token}")).map_err(|_| bad())?;
                v.set_sensitive(true);
                h.insert(AUTHORIZATION, v);
            }
        }
        Ok(h)
    }

    async fn post_text(&self, path: &str, body: &Value) -> Result<String, ModelError> {
        let req = self
            .client
            .post(format!("{}{path}", self.base_url))
            .headers(self.headers().await?)
            .json(body);
        http::send(&self.who, req, &ERRORS).await
    }

    async fn post_bytes(&self, path: &str, body: &Value) -> Result<Vec<u8>, ModelError> {
        let req = self
            .client
            .post(format!("{}{path}", self.base_url))
            .headers(self.headers().await?)
            .json(body);
        http::send_bytes(&self.who, req, &ERRORS).await
    }
}

fn env_key(var: &str, at: &Path) -> Result<String, Error> {
    std::env::var(var).map_err(|_| {
        Error::new(
            Phase::Interpretation,
            at.clone(),
            Rule::EnvMissing,
            format!("environment variable `{var}` is not set"),
        )
    })
}

#[async_trait]
impl Llm for OpenAi {
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
        if let Err(mut p) = self.media_kind(&req.output_media) {
            problems.append(&mut p);
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(self.who.unsupported(problems))
        }
    }

    async fn invoke(&self, call: &ModelInvocation) -> Result<ModelResponse, ModelError> {
        if let Some(kind) = self
            .media_kind(&call.output_media)
            .map_err(|p| self.who.unsupported(p))?
        {
            return self.invoke_media(call, kind).await;
        }
        let body = self.request_body(call)?;
        let (text, secs) = timed(self.post_text("/responses", &body)).await;
        let value: Value =
            serde_json::from_str(&text?).map_err(|e| self.who.invalid(format!("the answer is not JSON: {e}")))?;
        self.parse_response(call.schema.is_some(), &value, secs)
    }

    async fn invoke_batch(&self, requests: Vec<BatchRequest>, events: BatchEvents) -> BatchReturn {
        self.run_batch(requests, events).await
    }

    async fn cancel_batch(&self, batch_id: &str) -> Result<(), ModelError> {
        self.cancel(batch_id).await
    }
}
