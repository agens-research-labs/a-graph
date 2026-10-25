//! The Message Batches API: create, poll, read the results. One round only.

use std::collections::HashMap;

use serde_json::{Value, json};

use super::Anthropic;
use crate::models::{BatchEvent, BatchEvents, BatchOutcome, BatchRequest, BatchResult, BatchReturn, ModelError};

impl Anthropic {
    fn provider_error(&self, message: impl Into<String>) -> ModelError {
        self.who.provider_error(None, message)
    }

    pub(super) async fn run_batch(&self, requests: Vec<BatchRequest>, events: BatchEvents) -> BatchReturn {
        let mut results: Vec<BatchResult> = Vec::new();
        let mut sent = Vec::new(); // (id, wants a schema)
        let mut items = Vec::new();
        for r in requests {
            match self.request_body(&r.invocation) {
                Ok(params) => {
                    sent.push((r.id.clone(), r.invocation.schema.is_some()));
                    items.push(json!({ "custom_id": r.id, "params": params }));
                }
                // a request that cannot even be built does not stop the others
                Err(e) => results.push(BatchResult { id: r.id, result: Err(e) }),
            }
        }
        if items.is_empty() {
            return Ok(results);
        }

        let created = self.send(self.client.post(format!("{}/v1/messages/batches", self.base_url)).json(&json!({ "requests": items }))).await?;
        let created: Value = serde_json::from_str(&created).map_err(|e| self.invalid(format!("the batch answer is not JSON: {e}")))?;
        let batch_id = created.get("id").and_then(Value::as_str).ok_or_else(|| self.invalid("the batch answer has no `id`"))?.to_string();
        let _ = events.send(BatchEvent::Submitted { batch_id: batch_id.clone(), requests: sent.len() });

        // polled until Anthropic says `ended`: there is no maximum wait
        let counts = loop {
            let status = self.send(self.client.get(format!("{}/v1/messages/batches/{batch_id}", self.base_url))).await?;
            let status: Value = serde_json::from_str(&status).map_err(|e| self.invalid(format!("the batch status is not JSON: {e}")))?;
            let state = status.get("processing_status").and_then(Value::as_str).ok_or_else(|| self.invalid("the batch status has no `processing_status`"))?.to_string();
            let _ = events.send(BatchEvent::Status { batch_id: batch_id.clone(), status: state.clone() });
            if state == "ended" {
                break status.get("request_counts").cloned().unwrap_or(Value::Null);
            }
            tokio::time::sleep(self.poll_interval).await;
        };
        let n = |k: &str| counts.get(k).and_then(Value::as_u64).unwrap_or(0);
        let outcome = if n("succeeded") > 0 || sent.is_empty() {
            BatchOutcome::Completed
        } else if n("expired") > 0 {
            BatchOutcome::Expired
        } else {
            BatchOutcome::Failed
        };
        let _ = events.send(BatchEvent::Finished { batch_id: batch_id.clone(), outcome });

        // the results come from the documented path, never from a URL the server hands back:
        // the API key goes to the configured host only
        let body = self.send(self.client.get(format!("{}/v1/messages/batches/{batch_id}/results", self.base_url))).await?;
        let mut by_id: HashMap<String, Value> = HashMap::new();
        for line in body.lines().filter(|l| !l.trim().is_empty()) {
            let v: Value = serde_json::from_str(line).map_err(|e| self.invalid(format!("a result line is not JSON: {e}")))?;
            if let Some(id) = v.get("custom_id").and_then(Value::as_str) {
                by_id.insert(id.to_string(), v);
            }
        }

        for (id, schema) in sent {
            let result = match by_id.remove(&id) {
                None => Err(self.invalid(format!("the batch has no result for request `{id}`"))),
                Some(line) => self.batch_result(schema, &line["result"]),
            };
            results.push(BatchResult { id, result });
        }
        Ok(results)
    }

    fn batch_result(&self, schema: bool, result: &Value) -> Result<crate::models::ModelResponse, ModelError> {
        match result.get("type").and_then(Value::as_str) {
            // the time of one request inside a batch is not measurable: 0
            Some("succeeded") => self.parse_message(schema, &result["message"], 0.0),
            Some("errored") => {
                let message = result.pointer("/error/error/message").and_then(Value::as_str).unwrap_or("the request failed");
                Err(self.provider_error(format!("batch request failed: {message}")))
            }
            Some("expired") => Err(self.provider_error("the batch expired before this request could run")),
            Some("canceled") => Err(self.provider_error("the batch was canceled before this request could run")),
            other => Err(self.invalid(format!("unknown batch result type {other:?}"))),
        }
    }
}
