//! The `models` block: what the yml says about each model, read with the names of `schema::key`,
//! and the rules about its content. Building the providers from it is `ModelManager`'s job.

use serde::Deserialize;
use serde_norway::Value;

use super::reasoning::Modality;
use crate::build::{Errs, Obj};
use crate::error::{Path, Rule};
use crate::schema::key;

/// The value of `provider`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Provider {
    Anthropic,
    Openai,
    Gemini,
    Mistral,
    Bedrock,
    VertexAi,
    AzureFoundry,
    Ollama,
    Custom,
}

/// A secret is the NAME of an environment variable, never its value.
pub type EnvVar = String;

#[derive(Debug, Clone)]
pub enum Model {
    Anthropic(AnthropicModel),
    Openai(ApiModel),
    Gemini(ApiModel),
    Mistral(ApiModel),
    Bedrock(BedrockModel),
    VertexAi(VertexModel),
    AzureFoundry(AzureModel),
    Ollama(LocalModel),
    Custom(LocalModel),
}

/// What every model has.
#[derive(Debug, Clone)]
pub struct Common {
    pub id: String,
    pub model: String,
    pub input: Vec<Modality>,
    pub output: Vec<Modality>,
}

#[derive(Debug, Clone)]
pub struct AnthropicModel {
    pub common: Common,
    pub api_key: EnvVar,
    /// Sent as `anthropic-workspace-id`: needed by an API key not scoped to a workspace.
    pub workspace_id: Option<String>,
    /// The server the calls go to instead of the API of Anthropic: a gateway, or a proxy.
    pub host: Option<String>,
}

/// OpenAI, Gemini and Mistral: an API key and, if wanted, another server.
#[derive(Debug, Clone)]
pub struct ApiModel {
    pub common: Common,
    pub api_key: EnvVar,
    /// The server the calls go to instead of the API of the provider: a gateway, or a proxy.
    pub host: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BedrockModel {
    pub common: Common,
    pub region: String,
    pub credentials: BedrockCredentials,
    pub batch: Option<BedrockBatch>,
}

#[derive(Debug, Clone)]
pub enum BedrockCredentials {
    Ambient,
    Keys {
        access_key: EnvVar,
        secret_key: EnvVar,
        session_token: Option<EnvVar>,
    },
}

#[derive(Debug, Clone)]
pub struct BedrockBatch {
    pub input_s3_uri: String,
    pub output_s3_uri: String,
    pub role_arn: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VertexPublisher {
    Anthropic,
    Google,
}

#[derive(Debug, Clone)]
pub struct VertexModel {
    pub common: Common,
    pub publisher: VertexPublisher,
    pub project: String,
    pub location: String,
    pub batch: Option<VertexBatch>,
}

#[derive(Debug, Clone)]
pub struct VertexBatch {
    pub input_uri: String,
    pub output_uri_prefix: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AzurePublisher {
    Anthropic,
    Openai,
}

#[derive(Debug, Clone)]
pub struct AzureModel {
    pub common: Common,
    pub publisher: AzurePublisher,
    pub resource: String,
    pub credentials: AzureCredentials,
    pub batch: Option<AzureBatch>,
}

#[derive(Debug, Clone)]
pub enum AzureCredentials {
    Ambient,
    ApiKey(EnvVar),
}

#[derive(Debug, Clone)]
pub struct AzureBatch {
    pub deployment: String,
}

#[derive(Debug, Clone)]
pub struct LocalModel {
    pub common: Common,
    pub host: String,
}

/// The family of vendor fields a produced media can carry in its `provider`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Openai,
    Gemini,
}

impl Model {
    pub fn common(&self) -> &Common {
        match self {
            Model::Anthropic(m) => &m.common,
            Model::Openai(m) | Model::Gemini(m) | Model::Mistral(m) => &m.common,
            Model::Bedrock(m) => &m.common,
            Model::VertexAi(m) => &m.common,
            Model::AzureFoundry(m) => &m.common,
            Model::Ollama(m) | Model::Custom(m) => &m.common,
        }
    }

    pub fn id(&self) -> &str {
        &self.common().id
    }

    /// Configured provider identity, available even when an invocation never returns.
    pub fn provider_name(&self) -> &'static str {
        match self {
            Self::Anthropic(_) => "anthropic",
            Self::Openai(_) => "openai",
            Self::Gemini(_) => "gemini",
            Self::Mistral(_) => "mistral",
            Self::Bedrock(_) => "bedrock",
            Self::VertexAi(_) => "vertex_ai",
            Self::AzureFoundry(_) => "azure_foundry",
            Self::Ollama(_) => "ollama",
            Self::Custom(_) => "custom",
        }
    }

    pub fn output(&self) -> &[Modality] {
        &self.common().output
    }

    /// The family of the media `provider` fields this model accepts, if it produces media.
    pub fn family(&self) -> Option<Family> {
        match self {
            Model::Openai(_) => Some(Family::Openai),
            Model::AzureFoundry(a) if a.publisher == AzurePublisher::Openai => Some(Family::Openai),
            Model::Gemini(_) => Some(Family::Gemini),
            Model::VertexAi(v) if v.publisher == VertexPublisher::Google => Some(Family::Gemini),
            _ => None,
        }
    }

    /// Anthropic models (also on Vertex AI and Azure Foundry) refuse a call without `max_tokens`.
    pub fn requires_max_tokens(&self) -> bool {
        match self {
            Model::Anthropic(_) => true,
            Model::VertexAi(v) => v.publisher == VertexPublisher::Anthropic,
            Model::AzureFoundry(a) => a.publisher == AzurePublisher::Anthropic,
            _ => false,
        }
    }
}

/// Reads one item of the `models` list and checks its content. `None` when it cannot be built.
pub(super) fn read(mut o: Obj, errs: &mut Errs) -> Option<Model> {
    let at = o.at().clone();
    let before = errs.0.len();
    let provider = o.req::<Provider>(key::PROVIDER, errs);
    let common = common(&mut o, errs);

    let model = match (provider?, common?) {
        (Provider::Anthropic, common) => {
            let api_key = o.req_secret(key::API_KEY, errs);
            let workspace_id = o.opt::<String>(key::WORKSPACE_ID, errs);
            let host = o.opt::<String>(key::HOST, errs);
            Model::Anthropic(AnthropicModel {
                common,
                api_key: api_key?,
                workspace_id,
                host,
            })
        }
        (Provider::Openai, common) => {
            let api_key = o.req_secret(key::API_KEY, errs);
            let host = o.opt::<String>(key::HOST, errs);
            Model::Openai(ApiModel {
                common,
                api_key: api_key?,
                host,
            })
        }
        (Provider::Gemini, common) => {
            let api_key = o.req_secret(key::API_KEY, errs);
            let host = o.opt::<String>(key::HOST, errs);
            Model::Gemini(ApiModel {
                common,
                api_key: api_key?,
                host,
            })
        }
        (Provider::Mistral, common) => {
            let api_key = o.req_secret(key::API_KEY, errs);
            let host = o.opt::<String>(key::HOST, errs);
            Model::Mistral(ApiModel {
                common,
                api_key: api_key?,
                host,
            })
        }
        (Provider::Bedrock, common) => {
            let region = o.req::<String>(key::REGION, errs);
            let credentials = o
                .value(key::CREDENTIALS)
                .and_then(|v| bedrock_credentials(v, &at.key(key::CREDENTIALS), errs));
            let batch = o.sub(key::BATCH, errs).and_then(|b| bedrock_batch(b, errs));
            Model::Bedrock(BedrockModel {
                common,
                region: region?,
                credentials: credentials?,
                batch,
            })
        }
        (Provider::VertexAi, common) => {
            let publisher = o.req::<VertexPublisher>(key::PUBLISHER, errs);
            let project = o.req::<String>(key::PROJECT, errs);
            let location = o.req::<String>(key::LOCATION, errs);
            // `credentials: ambient` is the only option for now; the Loader checks the word
            let batch = o.sub(key::BATCH, errs).and_then(|mut b| {
                let (i, p) = (
                    b.req::<String>(key::INPUT_URI, errs),
                    b.req::<String>(key::OUTPUT_URI_PREFIX, errs),
                );
                Some(VertexBatch {
                    input_uri: i?,
                    output_uri_prefix: p?,
                })
            });
            Model::VertexAi(VertexModel {
                common,
                publisher: publisher?,
                project: project?,
                location: location?,
                batch,
            })
        }
        (Provider::AzureFoundry, common) => {
            let publisher = o.req::<AzurePublisher>(key::PUBLISHER, errs);
            let resource = o.req::<String>(key::RESOURCE, errs);
            let credentials = o
                .value(key::CREDENTIALS)
                .and_then(|v| azure_credentials(v, &at.key(key::CREDENTIALS), errs));
            let batch = o.sub(key::BATCH, errs).and_then(|mut b| {
                Some(AzureBatch {
                    deployment: b.req::<String>(key::DEPLOYMENT, errs)?,
                })
            });
            Model::AzureFoundry(AzureModel {
                common,
                publisher: publisher?,
                resource: resource?,
                credentials: credentials?,
                batch,
            })
        }
        (Provider::Ollama, common) => Model::Ollama(LocalModel {
            common,
            host: o.req::<String>(key::HOST, errs)?,
        }),
        (Provider::Custom, common) => Model::Custom(LocalModel {
            common,
            host: o.req::<String>(key::HOST, errs)?,
        }),
    };

    rules(&model, &at, errs);
    (errs.0.len() == before).then_some(model)
}

fn common(o: &mut Obj, errs: &mut Errs) -> Option<Common> {
    let (id, model) = (o.req::<String>(key::ID, errs), o.req::<String>(key::MODEL, errs));
    let (input, output) = (
        o.req::<Vec<Modality>>(key::INPUT, errs),
        o.req::<Vec<Modality>>(key::OUTPUT, errs),
    );
    Some(Common {
        id: id?,
        model: model?,
        input: input?,
        output: output?,
    })
}

/// `ambient`, or `{ access_key, secret_key, session_token }`.
fn bedrock_credentials(v: Value, at: &Path, errs: &mut Errs) -> Option<BedrockCredentials> {
    if v.as_str() == Some("ambient") {
        return Some(BedrockCredentials::Ambient);
    }
    let mut o = Obj::new(v, at.clone(), errs)?;
    let (access, secret) = (o.req_secret(key::ACCESS_KEY, errs), o.req_secret(key::SECRET_KEY, errs));
    let session_token = o.opt_secret(key::SESSION_TOKEN, errs);
    Some(BedrockCredentials::Keys {
        access_key: access?,
        secret_key: secret?,
        session_token,
    })
}

/// `ambient`, or `{ api_key }`.
fn azure_credentials(v: Value, at: &Path, errs: &mut Errs) -> Option<AzureCredentials> {
    if v.as_str() == Some("ambient") {
        return Some(AzureCredentials::Ambient);
    }
    let mut o = Obj::new(v, at.clone(), errs)?;
    Some(AzureCredentials::ApiKey(o.req_secret(key::API_KEY, errs)?))
}

fn bedrock_batch(mut b: Obj, errs: &mut Errs) -> Option<BedrockBatch> {
    let (i, o) = (
        b.req::<String>(key::INPUT_S3_URI, errs),
        b.req::<String>(key::OUTPUT_S3_URI, errs),
    );
    let role = b.req::<String>(key::ROLE_ARN, errs);
    Some(BedrockBatch {
        input_s3_uri: i?,
        output_s3_uri: o?,
        role_arn: role?,
    })
}

/// Rules about the content of one model.
fn rules(m: &Model, at: &Path, errs: &mut Errs) {
    let text_only = m.output().iter().all(|x| *x == Modality::Text);
    match m {
        Model::Bedrock(_) if !text_only => errs.add(
            &at.key(key::OUTPUT),
            Rule::OutputNotText,
            "media output is text only for now",
        ),
        Model::VertexAi(v) if v.publisher == VertexPublisher::Anthropic => {
            if !text_only {
                errs.add(
                    &at.key(key::OUTPUT),
                    Rule::OutputNotText,
                    "publisher anthropic has no media output",
                );
            }
            if v.batch.is_some() {
                errs.add(
                    &at.key(key::BATCH),
                    Rule::BatchNotAllowed,
                    "batch is only for publisher google",
                );
            }
        }
        Model::AzureFoundry(a) if a.publisher == AzurePublisher::Anthropic => {
            if !text_only {
                errs.add(
                    &at.key(key::OUTPUT),
                    Rule::OutputNotText,
                    "publisher anthropic has no media output",
                );
            }
            if a.batch.is_some() {
                errs.add(
                    &at.key(key::BATCH),
                    Rule::BatchNotAllowed,
                    "batch is only for publisher openai",
                );
            }
        }
        _ => {}
    }
}
