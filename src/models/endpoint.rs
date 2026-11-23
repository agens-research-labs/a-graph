//! The hosts each model is reached at: what the network allow-list checks.

use super::bedrock::hosts as bedrock;
use super::spec::{
    AnthropicModel, ApiModel, AzureCredentials, AzureModel, AzurePublisher, BedrockCredentials, BedrockModel, Model,
};
use crate::error::Path;
use crate::net::host_of;
use crate::network::Destination;
use crate::schema::key;
use crate::secrets::Secret;

impl Model {
    /// The URL written in the yml for the server of the model: `host` of a local server, or of a cloud provider
    /// reached through a gateway. `None` when the host is the one of the provider.
    fn url(&self) -> Option<&str> {
        match self {
            Model::Anthropic(AnthropicModel { host, .. })
            | Model::Openai(ApiModel { host, .. })
            | Model::Gemini(ApiModel { host, .. })
            | Model::Mistral(ApiModel { host, .. }) => host.as_deref(),
            Model::Ollama(l) | Model::Custom(l) => Some(&l.host),
            Model::Bedrock(_) | Model::VertexAi(_) | Model::AzureFoundry(_) => None,
        }
    }

    /// The host of the model endpoint, lowercase. `Err` carries the reason when it cannot be known.
    pub fn host(&self) -> Result<String, String> {
        if let Some(url) = self.url() {
            return host_of(url).ok_or_else(|| format!("`{url}` is not a URL with a host"));
        }
        // the host of the provider: a model with a URL (`host`) has returned above
        Ok(match self {
            Model::Anthropic(_) => "api.anthropic.com".into(),
            Model::Openai(_) => "api.openai.com".into(),
            Model::Gemini(_) => "generativelanguage.googleapis.com".into(),
            Model::Mistral(_) => "api.mistral.ai".into(),
            Model::Bedrock(b) => bedrock::runtime_host(&b.region)?,
            Model::VertexAi(v) => match v.location.to_lowercase().as_str() {
                "global" => "aiplatform.googleapis.com".into(),
                loc @ ("us" | "eu") => format!("aiplatform.{loc}.rep.googleapis.com"),
                loc => format!("{loc}-aiplatform.googleapis.com"),
            },
            Model::AzureFoundry(a) => match a.publisher {
                AzurePublisher::Anthropic => format!("{}.services.ai.azure.com", a.resource.to_lowercase()),
                AzurePublisher::Openai => format!("{}.openai.azure.com", a.resource.to_lowercase()),
            },
            Model::Ollama(_) | Model::Custom(_) => unreachable!("a local server always has a URL"),
        })
    }

    /// Primary configured destination, available before invocation (also for SDK-backed models).
    /// Only host and effective port are returned; URL credentials, path and query are never retained.
    pub(crate) fn server(&self) -> Option<(String, u16)> {
        let (host, port) = match self.url() {
            Some(url) => crate::net::host_port_of(url)?,
            None => (self.host().ok()?, Some(443)),
        };
        Some((host.trim_start_matches('[').trim_end_matches(']').to_string(), port?))
    }

    /// Every destination of the model, `at` being where it is in the yml: its endpoint and, for a
    /// Bedrock batch, the control API and the S3 buckets. The other batches use the endpoint.
    pub fn destinations(&self, at: &Path) -> Vec<Destination> {
        let mut out = vec![match self.url() {
            // a URL written in the yml (a local server, a gateway): its port counts
            Some(url) => Destination::url(at.clone(), url),
            None => Destination::host(at.clone(), self.host()),
        }];
        if let Model::Bedrock(BedrockModel {
            region, batch: Some(b), ..
        }) = self
        {
            let at = at.key(key::BATCH);
            out.push(Destination::host(at.clone(), bedrock::control_host(region)));
            out.push(Destination::host(
                at.key(key::INPUT_S3_URI),
                bedrock::s3_host(&b.input_s3_uri, region),
            ));
            out.push(Destination::host(
                at.key(key::OUTPUT_S3_URI),
                bedrock::s3_host(&b.output_s3_uri, region),
            ));
        }
        out
    }

    /// The `{ env }` of the model, `at` being where it is in the yml: each key goes to every
    /// destination of the model. `ambient` credentials name no variable.
    pub fn secrets(&self, at: &Path) -> Vec<Secret> {
        let to = self.destinations(at);
        let secret = |path: Path, var: &str| Secret {
            path,
            var: var.to_string(),
            destinations: to.clone(),
        };
        match self {
            Model::Anthropic(AnthropicModel { api_key, .. })
            | Model::Openai(ApiModel { api_key, .. })
            | Model::Gemini(ApiModel { api_key, .. })
            | Model::Mistral(ApiModel { api_key, .. }) => vec![secret(at.key(key::API_KEY), api_key)],
            Model::AzureFoundry(AzureModel {
                credentials: AzureCredentials::ApiKey(var),
                ..
            }) => vec![secret(at.key(key::CREDENTIALS).key(key::API_KEY), var)],
            Model::Bedrock(BedrockModel {
                credentials:
                    BedrockCredentials::Keys {
                        access_key,
                        secret_key,
                        session_token,
                    },
                ..
            }) => {
                let c = at.key(key::CREDENTIALS);
                let mut out = vec![
                    secret(c.key(key::ACCESS_KEY).key(key::ENV), access_key),
                    secret(c.key(key::SECRET_KEY).key(key::ENV), secret_key),
                ];
                if let Some(t) = session_token {
                    out.push(secret(c.key(key::SESSION_TOKEN).key(key::ENV), t));
                }
                out
            }
            Model::AzureFoundry(_) | Model::Bedrock(_) | Model::VertexAi(_) | Model::Ollama(_) | Model::Custom(_) => {
                vec![]
            }
        }
    }
}
