//! Ollama through `POST /api/embed`: `{model, input: [texts]}` answers `{embeddings: [[...]]}`.

use async_trait::async_trait;
use serde_json::{Value, json};

use super::Embedder;
use crate::error::{Error, Path, Phase, Rule};
use crate::models::{Identity, ModelError, http_client, http_send};

pub struct OllamaEmbedding {
    who: Identity,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl OllamaEmbedding {
    /// `id` and `host` are those of the yml; `at` is where the entry is.
    pub fn new(id: &str, model: &str, host: &str, at: &Path) -> Result<Self, Error> {
        let client = http_client().map_err(|e| {
            Error::new(
                Phase::Interpretation,
                at.clone(),
                Rule::Io,
                format!("cannot build the HTTP client: {e}"),
            )
        })?;
        Ok(OllamaEmbedding {
            who: Identity::new(id, "ollama"),
            model: model.to_string(),
            base_url: host.trim_end_matches('/').to_string(),
            client,
        })
    }

    fn api_error(&self, status: u16, body: &str) -> ModelError {
        let message = serde_json::from_str::<Value>(body)
            .ok()
            .and_then(|v| v.get("error").and_then(Value::as_str).map(str::to_string))
            .unwrap_or_else(|| body.chars().take(500).collect());
        self.who.provider_error(Some(status), message)
    }
}

#[async_trait]
impl Embedder for OllamaEmbedding {
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ModelError> {
        let req = self
            .client
            .post(format!("{}/api/embed", self.base_url))
            .json(&json!({ "model": self.model, "input": texts }));
        let text = http_send(&self.who, req, |status, body| self.api_error(status, body)).await?;
        let value: Value =
            serde_json::from_str(&text).map_err(|e| self.who.invalid(format!("the answer is not JSON: {e}")))?;
        let vectors = value
            .get("embeddings")
            .and_then(Value::as_array)
            .ok_or_else(|| self.who.invalid("the answer has no `embeddings` list"))?
            .iter()
            .map(|v| {
                v.as_array()
                    .filter(|a| !a.is_empty())
                    .and_then(|a| {
                        a.iter()
                            .map(|n| n.as_f64().map(|n| n as f32))
                            .collect::<Option<Vec<f32>>>()
                    })
                    .ok_or_else(|| self.who.invalid("an embedding is not a non-empty list of numbers"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if vectors.len() != texts.len() {
            return Err(self.who.invalid(format!(
                "{} texts sent, {} embeddings received",
                texts.len(),
                vectors.len()
            )));
        }
        Ok(vectors)
    }
}
