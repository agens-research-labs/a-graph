//! The Batch API: upload a JSONL file, create the batch, poll, read the output file.

use std::collections::HashMap;
use std::time::Duration;

use serde_json::{Value, json};

use super::OpenAi;
use crate::models::{BatchEvent, BatchEvents, BatchOutcome, BatchRequest, BatchResult, BatchReturn, ModelError};

/// How often a batch is polled. There is no maximum wait: it ends when OpenAI says so.
pub(super) const POLL_INTERVAL: Duration = Duration::from_secs(30);

impl OpenAi {
    pub(super) async fn run_batch(&self, requests: Vec<BatchRequest>, events: BatchEvents) -> BatchReturn {
        let mut results: Vec<BatchResult> = Vec::new();
        let mut sent = Vec::new(); // (id, wants a schema)
        let mut lines = Vec::new();
        // on Azure the batch names its own (Global-Batch) deployment
        let model = self.batch_model.clone().unwrap_or_else(|| self.model.clone());
        for r in requests {
            let built = if r.invocation.output_media.is_empty() {
                self.request_body_for(&model, &r.invocation)
            } else {
                Err(self
                    .who
                    .unsupported(vec!["a produced media cannot go in a batch of responses".into()]))
            };
            match built {
                Ok(body) => {
                    sent.push((r.id.clone(), r.invocation.schema.is_some()));
                    lines.push(
                        json!({ "custom_id": r.id, "method": "POST", "url": "/v1/responses", "body": body })
                            .to_string(),
                    );
                }
                // a request that cannot even be built does not stop the others
                Err(e) => results.push(BatchResult {
                    id: r.id,
                    result: Err(e),
                }),
            }
        }
        if lines.is_empty() {
            return Ok(results);
        }

        let part = reqwest::multipart::Part::text(lines.join("\n") + "\n")
            .file_name("batch.jsonl")
            .mime_str("application/jsonl")
            .map_err(|e| self.who.network(e))?;
        let form = reqwest::multipart::Form::new()
            .text("purpose", "batch")
            .part("file", part);
        let req = self
            .client
            .post(format!("{}/files", self.base_url))
            .headers(self.headers().await?)
            .multipart(form);
        let file = self.parse_json(crate::models::http::send(&self.who, req, |s, b| self.api_error(s, b)).await?)?;
        let file_id = file
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| self.who.invalid("the uploaded file has no `id`"))?
            .to_string();

        let created = self
            .post_text(
                "/batches",
                &json!({ "input_file_id": file_id, "endpoint": "/v1/responses", "completion_window": "24h" }),
            )
            .await?;
        let created = self.parse_json(created)?;
        let batch_id = created
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| self.who.invalid("the batch has no `id`"))?
            .to_string();
        let _ = events.send(BatchEvent::Submitted {
            batch_id: batch_id.clone(),
            requests: sent.len(),
        });

        // polled until a terminal state: there is no maximum wait
        let last = loop {
            let status = self.get_text(&format!("/batches/{batch_id}")).await?;
            let status = self.parse_json(status)?;
            let state = status
                .get("status")
                .and_then(Value::as_str)
                .ok_or_else(|| self.who.invalid("the batch has no `status`"))?
                .to_string();
            let _ = events.send(BatchEvent::Status {
                batch_id: batch_id.clone(),
                status: state.clone(),
            });
            if matches!(state.as_str(), "completed" | "failed" | "expired" | "cancelled") {
                break status;
            }
            tokio::time::sleep(self.poll_interval).await;
        };
        let state = last["status"].as_str().unwrap_or_default().to_string();
        let outcome = match state.as_str() {
            "completed" => BatchOutcome::Completed,
            "expired" => BatchOutcome::Expired,
            _ => BatchOutcome::Failed,
        };
        let _ = events.send(BatchEvent::Finished {
            batch_id: batch_id.clone(),
            outcome,
        });

        let output_id = last.get("output_file_id").and_then(Value::as_str);
        let error_id = last.get("error_file_id").and_then(Value::as_str);
        if output_id.is_none() && error_id.is_none() {
            let why = last
                .pointer("/errors/data/0/message")
                .and_then(Value::as_str)
                .unwrap_or("no result file");
            return Err(self.who.provider_error(None, format!("batch {state}: {why}")));
        }

        let mut by_id: HashMap<String, Value> = HashMap::new();
        for file_id in [output_id, error_id].into_iter().flatten() {
            for line in self
                .get_text(&format!("/files/{file_id}/content"))
                .await?
                .lines()
                .filter(|l| !l.trim().is_empty())
            {
                let v: Value = serde_json::from_str(line)
                    .map_err(|e| self.who.invalid(format!("a result line is not JSON: {e}")))?;
                if let Some(id) = v.get("custom_id").and_then(Value::as_str) {
                    by_id.insert(id.to_string(), v);
                }
            }
        }

        for (id, schema) in sent {
            let result = match by_id.remove(&id) {
                None => Err(self.who.invalid(format!("the batch has no result for request `{id}`"))),
                Some(line) => self.batch_result(schema, &line),
            };
            results.push(BatchResult { id, result });
        }
        Ok(results)
    }

    fn batch_result(&self, schema: bool, line: &Value) -> Result<crate::models::ModelResponse, ModelError> {
        if let Some(err) = line.get("error").filter(|e| !e.is_null()) {
            let msg = err
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("the request failed");
            return Err(self.who.provider_error(None, format!("batch request failed: {msg}")));
        }
        let status = line
            .pointer("/response/status_code")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let body = &line["response"]["body"];
        if status != 200 {
            let msg = body
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("the request failed");
            return Err(self.who.provider_error(Some(status as u16), msg));
        }
        // the time of one request inside a batch is not measurable: 0
        self.parse_response(schema, body, 0.0)
    }

    /// `POST /batches/{id}/cancel`: the batch goes to `cancelling`, then `cancelled`.
    pub(super) async fn cancel(&self, batch_id: &str) -> Result<(), ModelError> {
        self.post_text(&format!("/batches/{batch_id}/cancel"), &json!({}))
            .await
            .map(drop)
    }

    fn parse_json(&self, text: String) -> Result<Value, ModelError> {
        serde_json::from_str(&text).map_err(|e| self.who.invalid(format!("the answer is not JSON: {e}")))
    }

    async fn get_text(&self, path: &str) -> Result<String, ModelError> {
        let req = self
            .client
            .get(format!("{}{path}", self.base_url))
            .headers(self.headers().await?);
        crate::models::http::send(&self.who, req, |s, b| self.api_error(s, b)).await
    }
}
