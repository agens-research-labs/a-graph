//! The Anthropic Messages API, called directly over REST: from Anthropic itself, from Vertex AI
//! (`:rawPredict`) and from Azure Foundry. The three speak the same format and differ in where the
//! request goes and how it proves who it is.
//!
//! No model table: the request is always built the same way, and what a given model does not
//! accept comes back as Anthropic's own error (for `reasoning`, for `temperature`...), which is
//! shown as it is. The models and their rules are described in the documentation.

mod batch;
mod request;
mod response;

use std::fmt;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;

use super::auth::{AzureAmbient, GoogleAdc, TokenSource};
use super::http::{self, ErrorShape, refused_credentials};
use super::identity::Identity;
use super::spec::{AnthropicModel, AzureCredentials, AzureModel, Model, VertexModel};
use super::timing::timed;
use super::{
    BatchEvents, BatchRequest, BatchReturn, Llm, ModelError, ModelInvocation, ModelRequirements, ModelResponse,
};
use crate::error::{Error, Path, Phase, Rule};
use crate::resources::MediaKind;
use crate::schema::key;

const VERSION: &str = "2023-06-01";
const BASE_URL: &str = "https://api.anthropic.com";
/// How often a batch is polled. There is no maximum wait: it ends when Anthropic says so.
const POLL_INTERVAL: Duration = Duration::from_secs(30);

/// Where the request goes: it changes the URL, the body and the way the call proves who it is.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Flavor {
    Direct,
    /// Vertex AI: no `model` in the body (it is in the URL), `anthropic_version` in the body.
    Vertex,
    /// Azure Foundry: `model` is the name of the deployment.
    Azure,
}

enum Auth {
    /// `x-api-key` (Anthropic), with the workspace when the key is not scoped to one.
    Key { key: String, workspace: Option<String> },
    /// `api-key` (Azure).
    AzureKey(String),
    /// `Authorization: Bearer <token>`; the token comes from ambient credentials.
    Bearer(Box<dyn TokenSource>),
}

/// `{ "error": { "type", "message" } }`: the type is the code.
const ERRORS: ErrorShape = ErrorShape {
    code: &["/error/type"],
    message: &["/error/message"],
    credentials: refused_credentials,
};

pub struct Anthropic {
    who: Identity,
    model: String,
    flavor: Flavor,
    auth: Auth,
    /// Direct: the server of `/v1/messages` and of the batches.
    base_url: String,
    /// Vertex and Azure: the whole URL of the call.
    messages_url: String,
    poll_interval: Duration,
    client: reqwest::Client,
}

impl fmt::Debug for Anthropic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Anthropic")
            .field("provider", &self.who.provider)
            .field("model", &self.model)
            .field("credentials", &"<redacted>")
            .finish()
    }
}

impl Anthropic {
    /// Anthropic itself. Reads the key from the environment variable the yml names; `at` is where
    /// the model is in the yml. A variable that is not set is an error now, not at the first call.
    pub fn new(m: &AnthropicModel, at: &Path) -> Result<Self, Error> {
        let key = env_key(&m.api_key, &at.key(key::API_KEY))?;
        let anthropic = Self::assemble(
            &m.common.id,
            &m.common.model,
            "anthropic",
            Flavor::Direct,
            Auth::Key {
                key,
                workspace: m.workspace_id.clone(),
            },
            String::new(),
            at,
        )?;
        Ok(match &m.host {
            Some(host) => anthropic.with_base_url(host),
            None => anthropic,
        })
    }

    /// Claude on Vertex AI (publisher anthropic), with the Google Application Default Credentials.
    pub fn vertex(m: &VertexModel, at: &Path) -> Result<Self, Error> {
        let host = Model::VertexAi(m.clone())
            .host()
            .map_err(|e| Error::new(Phase::Interpretation, at.clone(), Rule::BadValue, e))?;
        let url = format!(
            "https://{host}/v1/projects/{}/locations/{}/publishers/anthropic/models/{}:rawPredict",
            m.project, m.location, m.common.model
        );
        Self::assemble(
            &m.common.id,
            &m.common.model,
            "vertex_ai",
            Flavor::Vertex,
            Auth::Bearer(Box::new(GoogleAdc::new())),
            url,
            at,
        )
    }

    /// Claude on Azure Foundry (publisher anthropic): `model` of the yml is the name of the deployment.
    pub fn azure(m: &AzureModel, at: &Path) -> Result<Self, Error> {
        let auth = match &m.credentials {
            AzureCredentials::ApiKey(var) => Auth::AzureKey(env_key(var, &at.key(key::CREDENTIALS).key(key::API_KEY))?),
            AzureCredentials::Ambient => Auth::Bearer(Box::new(AzureAmbient::new("https://ai.azure.com/.default"))),
        };
        let url = format!(
            "https://{}.services.ai.azure.com/anthropic/v1/messages",
            m.resource.to_lowercase()
        );
        Self::assemble(
            &m.common.id,
            &m.common.model,
            "azure_foundry",
            Flavor::Azure,
            auth,
            url,
            at,
        )
    }

    fn assemble(
        id: &str,
        model: &str,
        provider: &'static str,
        flavor: Flavor,
        auth: Auth,
        messages_url: String,
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
        Ok(Anthropic {
            who: Identity::new(id, provider),
            model: model.to_string(),
            flavor,
            auth,
            base_url: BASE_URL.into(),
            messages_url,
            poll_interval: POLL_INTERVAL,
            client,
        })
    }

    /// Another server for the direct API, for the calls and the batches: the `host` of the yml, which the allow-list
    /// checks, or the server of a test.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into().trim_end_matches('/').to_string();
        self
    }

    /// For tests: another URL for the call of Vertex AI or Azure.
    pub fn with_messages_url(mut self, url: impl Into<String>) -> Self {
        self.messages_url = url.into();
        self
    }

    /// Another source of the bearer token (tests, or a token the caller already has).
    pub fn with_token_source(mut self, source: Box<dyn TokenSource>) -> Self {
        self.auth = Auth::Bearer(source);
        self
    }

    pub fn with_poll_interval(mut self, every: Duration) -> Self {
        self.poll_interval = every;
        self
    }

    /// The URL of the call (without the key or the token): the host the allow-list checked.
    pub fn endpoint(&self) -> String {
        self.messages_endpoint()
    }

    fn messages_endpoint(&self) -> String {
        match self.flavor {
            Flavor::Direct => format!("{}/v1/messages", self.base_url),
            _ => self.messages_url.clone(),
        }
    }

    async fn headers(&self) -> Result<HeaderMap, ModelError> {
        let bad = |what: &str| {
            self.who
                .credentials(format!("the {what} has characters not allowed in a header"))
        };
        let mut h = HeaderMap::new();
        match &self.auth {
            Auth::Key { key, workspace } => {
                let mut v = HeaderValue::from_str(key).map_err(|_| bad("API key"))?;
                v.set_sensitive(true);
                h.insert(HeaderName::from_static("x-api-key"), v);
                if let Some(ws) = workspace {
                    h.insert(
                        HeaderName::from_static("anthropic-workspace-id"),
                        HeaderValue::from_str(ws).map_err(|_| bad("workspace_id"))?,
                    );
                }
            }
            Auth::AzureKey(key) => {
                let mut v = HeaderValue::from_str(key).map_err(|_| bad("API key"))?;
                v.set_sensitive(true);
                h.insert(HeaderName::from_static("api-key"), v);
            }
            Auth::Bearer(source) => {
                let token = source.token().await.map_err(|detail| self.who.credentials(detail))?;
                let mut v = HeaderValue::from_str(&format!("Bearer {token}")).map_err(|_| bad("token"))?;
                v.set_sensitive(true);
                h.insert(reqwest::header::AUTHORIZATION, v);
            }
        }
        // on Vertex AI the version is in the body
        if self.flavor != Flavor::Vertex {
            h.insert(
                HeaderName::from_static("anthropic-version"),
                HeaderValue::from_static(VERSION),
            );
        }
        Ok(h)
    }

    fn unsupported(&self, problems: Vec<String>) -> ModelError {
        self.who.unsupported(problems)
    }

    fn invalid(&self, detail: impl Into<String>) -> ModelError {
        self.who.invalid(detail)
    }

    /// POST/GET and read the whole body; `Err` is already a `ModelError`.
    async fn send(&self, req: reqwest::RequestBuilder) -> Result<String, ModelError> {
        http::send(&self.who, req.headers(self.headers().await?), &ERRORS).await
    }
}

#[async_trait]
impl Llm for Anthropic {
    fn check(&self, req: &ModelRequirements) -> Result<(), ModelError> {
        let mut problems = Vec::new();
        if req.max_tokens.is_none() {
            problems.push("max_tokens is mandatory for anthropic".to_string());
        }
        if req.batch && self.flavor != Flavor::Direct {
            problems.push("Claude has no batch on this platform".to_string());
        }
        if !req.output_media.is_empty() {
            problems.push("anthropic produces text only".to_string());
        }
        if req.logprobs.is_some() {
            problems.push("anthropic gives no logprobs".to_string());
        }
        for t in &req.input_media {
            match t {
                MediaKind::Md => {}
                MediaKind::Img(f) if request::image_mime(f).is_some() => {}
                MediaKind::Img(f) => {
                    problems.push(format!("image format `{f}` is not accepted (jpeg, png, gif, webp)"))
                }
                MediaKind::File => problems.push(
                    "a file of any format cannot go to a model: a utility has to change it into text first".to_string(),
                ),
                MediaKind::Audio(_) | MediaKind::Video(_) => {
                    problems.push("anthropic accepts text and images only".to_string())
                }
            }
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(self.unsupported(problems))
        }
    }

    async fn invoke(&self, call: &ModelInvocation) -> Result<ModelResponse, ModelError> {
        let body = self.request_body(call)?;
        let (text, secs) = timed(self.send(self.client.post(self.messages_endpoint()).json(&body))).await;
        let value: Value =
            serde_json::from_str(&text?).map_err(|e| self.invalid(format!("the answer is not JSON: {e}")))?;
        self.parse_message(&value, secs)
    }

    async fn invoke_batch(&self, requests: Vec<BatchRequest>, events: BatchEvents) -> BatchReturn {
        if self.flavor != Flavor::Direct {
            return Err(self.unsupported(vec!["Claude has no batch on this platform".into()]));
        }
        self.run_batch(requests, events).await
    }

    async fn cancel_batch(&self, batch_id: &str) -> Result<(), ModelError> {
        self.cancel(batch_id).await
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
