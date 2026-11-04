//! The batch jobs: the requests go inline in the job, the results come back in a file.

use std::collections::HashMap;

use serde_json::{Value, json};

use super::Mistral;
use crate::models::{BatchEvent, BatchEvents, BatchOutcome, BatchRequest, BatchResult, BatchReturn, ModelError};

impl Mistral {
    pub(super) async fn run_batch(&self, requests: Vec<BatchRequest>, events: BatchEvents) -> BatchReturn {
        let mut results: Vec<BatchResult> = Vec::new();
        let mut sent = Vec::new(); // (id, wants a schema)
        let mut items = Vec::new();
        for r in requests {
            let built = if r.invocation.output_media.is_empty() {
                self.request_body(&r.invocation)
            } else {
                Err(self
                    .who
                    .unsupported(vec!["a produced media cannot go in a batch of chat completions".into()]))
            };
            match built {
                Ok(body) => {
                    sent.push((r.id.clone(), r.invocation.schema.is_some()));
                    items.push(json!({ "custom_id": r.id, "body": body }));
                }
                // a request that cannot even be built does not stop the others
                Err(e) => results.push(BatchResult {
                    id: r.id,
                    result: Err(e),
                }),
            }
        }
        if items.is_empty() {
            return Ok(results);
        }

        let created = self
            .post_text(
                "/v1/batch/jobs",
                &json!({ "requests": items, "endpoint": "/v1/chat/completions", "model": self.model }),
            )
            .await?;
        let created = self.parse_json(created)?;
        let job_id = created
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| self.who.invalid("the batch job has no `id`"))?
            .to_string();
        let _ = events.send(BatchEvent::Submitted {
            batch_id: job_id.clone(),
            requests: sent.len(),
        });

        // polled until a terminal state: there is no maximum wait
        let last = loop {
            let job = self.parse_json(self.get_text(&format!("/v1/batch/jobs/{job_id}")).await?)?;
            let state = job
                .get("status")
                .and_then(Value::as_str)
                .ok_or_else(|| self.who.invalid("the batch job has no `status`"))?
                .to_string();
            let _ = events.send(BatchEvent::Status {
                batch_id: job_id.clone(),
                status: state.clone(),
            });
            if matches!(state.as_str(), "SUCCESS" | "FAILED" | "TIMEOUT_EXCEEDED" | "CANCELLED") {
                break job;
            }
            tokio::time::sleep(self.poll_interval).await;
        };
        let state = last["status"].as_str().unwrap_or_default().to_string();
        let outcome = match state.as_str() {
            "SUCCESS" => BatchOutcome::Completed,
            "TIMEOUT_EXCEEDED" => BatchOutcome::Expired,
            _ => BatchOutcome::Failed,
        };
        let _ = events.send(BatchEvent::Finished {
            batch_id: job_id.clone(),
            outcome,
        });

        let output = last.get("output_file").and_then(Value::as_str);
        let errors = last.get("error_file").and_then(Value::as_str);
        if output.is_none() && errors.is_none() {
            let why = last
                .pointer("/errors/0/message")
                .and_then(Value::as_str)
                .unwrap_or("no result file");
            return Err(self.who.provider_error(None, format!("batch {state}: {why}")));
        }

        let mut by_id: HashMap<String, Value> = HashMap::new();
        for file in [output, errors].into_iter().flatten() {
            for line in self
                .get_text(&format!("/v1/files/{file}/content"))
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
            .unwrap_or(200);
        let body = &line["response"]["body"];
        if status != 200 {
            let msg = body
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("the request failed");
            return Err(self.who.provider_error(Some(status as u16), msg));
        }
        // the time of one request inside a batch is not measurable: 0
        self.parse_completion(schema, body, 0.0)
    }

    /// `POST /v1/batch/jobs/{id}/cancel`: the job goes to `CANCELLATION_REQUESTED`, then `CANCELLED`.
    pub(super) async fn cancel(&self, job_id: &str) -> Result<(), ModelError> {
        self.post_text(&format!("/v1/batch/jobs/{job_id}/cancel"), &json!({}))
            .await
            .map(drop)
    }

    fn parse_json(&self, text: String) -> Result<Value, ModelError> {
        serde_json::from_str(&text).map_err(|e| self.who.invalid(format!("the answer is not JSON: {e}")))
    }
}
