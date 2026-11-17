//! Amazon Bedrock through the Converse API, with the official AWS SDK.
//!
//! The same call serves every model family Bedrock hosts. No model table: what a model does not
//! accept comes back as Bedrock's own error, shown as it is. Reasoning is sent the way Claude
//! models take it. The credentials are `ambient` (the SDK's default chain, resolved at the first
//! call) or explicit keys read from the environment at load.

mod batch;
mod convert;
pub(super) mod hosts;
mod request;
mod response;

use async_trait::async_trait;
use aws_config::SdkConfig;
use aws_sdk_bedrockruntime::Client;
use aws_sdk_bedrockruntime::config::{BehaviorVersion, Credentials, Region, SharedCredentialsProvider};
use aws_smithy_types::timeout::TimeoutConfig;
use std::fmt;
use tokio::sync::OnceCell;

use super::http::CALL_TIMEOUT;
use super::identity::Identity;
use super::spec::{BedrockBatch, BedrockCredentials, BedrockModel};
use super::timing::timed;
use super::{
    BatchEvents, BatchRequest, BatchReturn, Llm, ModelError, ModelInvocation, ModelRequirements, ModelResponse,
};
use crate::error::{Error, Path, Phase, Rule};
use crate::resources::MediaKind;
use crate::schema::key;

const PROVIDER: &str = "bedrock";

pub struct Bedrock {
    who: Identity,
    model: String,
    region: String,
    /// `None`: the SDK's default chain, resolved at the first call.
    keys: Option<Keys>,
    endpoint_url: Option<String>,
    /// The `batch` block of the yml.
    batch: Option<BedrockBatch>,
    poll_interval: std::time::Duration,
    call_timeout: std::time::Duration,
    config: OnceCell<SdkConfig>,
}

/// How often a batch job is polled. There is no maximum wait: it ends when Bedrock says so.
const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60);

struct Keys {
    access_key: String,
    secret_key: String,
    session_token: Option<String>,
}

impl fmt::Debug for Bedrock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bedrock")
            .field("model", &self.model)
            .field("region", &self.region)
            .field("credentials", &"<redacted>")
            .finish()
    }
}

fn env(var: &str, at: &Path) -> Result<String, Error> {
    std::env::var(var).map_err(|_| {
        Error::new(
            Phase::Interpretation,
            at.clone(),
            Rule::EnvMissing,
            format!("environment variable `{var}` is not set"),
        )
    })
}

impl Bedrock {
    /// `at` is where the model is in the yml. With explicit keys, a variable that is not set is an
    /// error now; with `ambient`, nothing is resolved until the first call.
    pub fn new(m: &BedrockModel, at: &Path) -> Result<Self, Error> {
        let keys = match &m.credentials {
            BedrockCredentials::Ambient => None,
            BedrockCredentials::Keys {
                access_key,
                secret_key,
                session_token,
            } => {
                let c = at.key(key::CREDENTIALS);
                Some(Keys {
                    access_key: env(access_key, &c.key(key::ACCESS_KEY).key(key::ENV))?,
                    secret_key: env(secret_key, &c.key(key::SECRET_KEY).key(key::ENV))?,
                    session_token: session_token
                        .as_ref()
                        .map(|t| env(t, &c.key(key::SESSION_TOKEN).key(key::ENV)))
                        .transpose()?,
                })
            }
        };
        Ok(Bedrock {
            who: Identity::new(&m.common.id, PROVIDER),
            model: m.common.model.clone(),
            region: m.region.clone(),
            keys,
            endpoint_url: None,
            batch: m.batch.clone(),
            poll_interval: POLL_INTERVAL,
            call_timeout: CALL_TIMEOUT,
            config: OnceCell::new(),
        })
    }

    /// For tests: another server. Not reachable from the yml, whose host the allow-list checks.
    pub fn with_endpoint_url(mut self, url: impl Into<String>) -> Self {
        self.endpoint_url = Some(url.into());
        self
    }

    pub fn with_poll_interval(mut self, every: std::time::Duration) -> Self {
        self.poll_interval = every;
        self
    }

    pub fn with_call_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.call_timeout = timeout;
        self
    }

    /// The SDK configuration, built at the first call: with `ambient` credentials this is where the
    /// default chain starts to be used. It is pinned to the hosts the allow-list checked.
    async fn sdk_config(&self) -> &SdkConfig {
        self.config
            .get_or_init(|| async {
                // the clients of Bedrock and S3 are built from this configuration: no call may hang for ever
                let mut loader = aws_config::defaults(BehaviorVersion::latest())
                    .region(Region::new(self.region.clone()))
                    .timeout_config(TimeoutConfig::builder().operation_timeout(self.call_timeout).build());
                if let Some(k) = &self.keys {
                    loader = loader.credentials_provider(SharedCredentialsProvider::new(Credentials::new(
                        &k.access_key,
                        &k.secret_key,
                        k.session_token.clone(),
                        None,
                        "a-graph",
                    )));
                }
                hosts::pinned(loader.load().await, self.endpoint_url.clone())
            })
            .await
    }

    async fn client(&self) -> Result<Client, ModelError> {
        Ok(Client::new(self.sdk_config().await))
    }
}

#[async_trait]
impl Llm for Bedrock {
    fn check(&self, req: &ModelRequirements) -> Result<(), ModelError> {
        let mut problems = Vec::new();
        if !req.output_media.is_empty() {
            problems.push("media output is text only for now".to_string());
        }
        if req.logprobs.is_some() {
            problems.push("the Converse API of Bedrock gives no logprobs".to_string());
        }
        if req.batch && req.schema.is_some() {
            problems.push("a batch on Bedrock cannot use an output schema".to_string());
        }
        if req.batch && self.batch.is_none() {
            problems.push("this model has no `batch` block in the yml".to_string());
        }
        for t in &req.input_media {
            match t {
                MediaKind::Md => {}
                MediaKind::Img(f) if convert::image_format(f).is_some() => {}
                MediaKind::Img(f) => {
                    problems.push(format!("image format `{f}` is not accepted (png, jpeg, gif, webp)"))
                }
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
        if !call.output_media.is_empty() {
            return Err(self.who.unsupported(vec!["media output is text only for now".into()]));
        }
        let client = self.client().await?;
        let (out, secs) = timed(self.converse(&client, call)).await;
        self.parse_converse(call.schema.is_some(), out?, secs)
    }

    async fn invoke_batch(&self, requests: Vec<BatchRequest>, events: BatchEvents) -> BatchReturn {
        self.run_batch(requests, events).await
    }

    async fn cancel_batch(&self, batch_id: &str) -> Result<(), ModelError> {
        self.cancel(batch_id).await
    }
}
