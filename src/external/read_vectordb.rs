//! Reading a vector database: the vector of the query goes in, the texts of the nearest chunks
//! come out, the nearest first.

use std::time::Duration;

use serde_json::{Value, json};

use super::{VectorBackend, Vectordb, VectordbManager};

/// A search is a short call: it must not hang a run. Seconds, when the yml has no `timeout`.
const DEFAULT_TIMEOUT: u64 = 60;
/// How many chunks come back when the yml does not say `top_k`.
const DEFAULT_TOP_K: u32 = 5;

impl VectordbManager {
    /// The texts of the chunks of database `id` nearest to `vector`. The error says what failed.
    pub async fn search(&self, id: &str, vector: &[f32]) -> Result<Vec<String>, String> {
        let db = self.get(id).ok_or_else(|| format!("vectordb `{id}` is not declared"))?;
        db.search(&self.client, vector).await
    }
}

impl Vectordb {
    async fn search(&self, client: &reqwest::Client, vector: &[f32]) -> Result<Vec<String>, String> {
        let top_k = self.top_k.unwrap_or(DEFAULT_TOP_K);
        let timeout = Duration::from_secs(self.timeout.map_or(DEFAULT_TIMEOUT, u64::from));
        let key = match &self.api_key {
            Some(var) => Some(
                std::env::var(var)
                    .map_err(|_| format!("vectordb `{}`: the environment variable `{var}` is not set", self.id))?,
            ),
            None => None,
        };
        match &self.backend {
            VectorBackend::Qdrant {
                url,
                collection,
                text_field,
            } => {
                let mut req = client
                    .post(format!(
                        "{}/collections/{collection}/points/query",
                        url.trim_end_matches('/')
                    ))
                    .json(&json!({
                        "query": vector,
                        "limit": top_k,
                        "with_payload": [text_field],
                    }));
                if let Some(key) = key {
                    req = with_secret(req, "api-key", key);
                }
                let answer = self.send(req, timeout).await?;
                let points = answer["result"]["points"]
                    .as_array()
                    .ok_or_else(|| format!("vectordb `{}`: the answer has no `result.points`", self.id))?;
                points
                    .iter()
                    .map(|p| {
                        p["payload"][text_field.as_str()]
                            .as_str()
                            .map(str::to_string)
                            .ok_or_else(|| {
                                format!(
                                    "vectordb `{}`: a point has no text in the payload field `{text_field}`",
                                    self.id
                                )
                            })
                    })
                    .collect()
            }
            VectorBackend::Chromadb {
                url,
                collection,
                tenant,
                database,
            } => {
                let base = format!(
                    "{}/api/v2/tenants/{tenant}/databases/{database}/collections",
                    url.trim_end_matches('/')
                );
                let with_key = |req: reqwest::RequestBuilder| match &key {
                    Some(key) => with_secret(req, "x-chroma-token", key.clone()),
                    None => req,
                };
                // the query is made on the id of the collection, which the name gives
                let found = self
                    .send(with_key(client.get(format!("{base}/{collection}"))), timeout)
                    .await?;
                let collection_id = found["id"]
                    .as_str()
                    .ok_or_else(|| format!("vectordb `{}`: the answer has no collection `id`", self.id))?;
                let answer = self
                    .send(
                        with_key(client.post(format!("{base}/{collection_id}/query"))).json(&json!({
                            "query_embeddings": [vector],
                            "n_results": top_k,
                            "include": ["documents"],
                        })),
                        timeout,
                    )
                    .await?;
                let documents = answer["documents"][0]
                    .as_array()
                    .ok_or_else(|| format!("vectordb `{}`: the answer has no `documents`", self.id))?;
                Ok(documents.iter().filter_map(Value::as_str).map(str::to_string).collect())
            }
        }
    }

    /// Sends the request and reads the JSON answer; a non-2xx status is an error with its body.
    async fn send(&self, req: reqwest::RequestBuilder, timeout: Duration) -> Result<Value, String> {
        let resp = req
            .timeout(timeout)
            .send()
            .await
            .map_err(|e| format!("vectordb `{}`: {e}", self.id))?;
        let status = resp.status();
        let body = resp.text().await.map_err(|e| format!("vectordb `{}`: {e}", self.id))?;
        if !status.is_success() {
            return Err(format!(
                "vectordb `{}`: the server answered {}: {}",
                self.id,
                status.as_u16(),
                body.chars().take(500).collect::<String>()
            ));
        }
        serde_json::from_str(&body).map_err(|e| format!("vectordb `{}`: the answer is not JSON: {e}", self.id))
    }
}

/// The request with a header that carries a secret, marked sensitive. A value that is not valid
/// is reported by `send`, as before.
fn with_secret(req: reqwest::RequestBuilder, name: &'static str, value: String) -> reqwest::RequestBuilder {
    match crate::net::secret_value(&value) {
        Ok(v) => req.header(name, v),
        Err(_) => req.header(name, value),
    }
}
