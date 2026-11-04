//! Batch inference: the requests go to S3 as a JSONL file in the Converse format, a model
//! invocation job runs them, the results are read from the output folder.

use std::collections::HashMap;

use aws_sdk_bedrock::types::{
    ModelInvocationJobInputDataConfig, ModelInvocationJobOutputDataConfig, ModelInvocationJobS3InputDataConfig,
    ModelInvocationJobS3OutputDataConfig, ModelInvocationJobStatus, ModelInvocationType, S3InputFormat,
};
use aws_sdk_s3::primitives::ByteStream;
use serde_json::{Map, Value, json};

use super::Bedrock;
use super::hosts::split_s3;
use crate::models::Reasoning;
use crate::models::{
    BatchEvent, BatchEvents, BatchOutcome, BatchRequest, BatchResult, BatchReturn, MediaInput, ModelError,
    ModelInvocation, ModelResponse, ResponseInfo, Role, StopReason, ToolCall, Usage,
};

impl Bedrock {
    pub(super) async fn run_batch(&self, requests: Vec<BatchRequest>, events: BatchEvents) -> BatchReturn {
        let Some(cfg) = &self.batch else {
            return Err(self
                .who
                .unsupported(vec!["this model has no `batch` block in the yml".into()]));
        };
        let bad_uri = |what: &str, uri: &str| {
            self.who
                .unsupported(vec![format!("{what} `{uri}` is not an s3:// URI")])
        };
        let (in_bucket, in_prefix) =
            split_s3(&cfg.input_s3_uri).ok_or_else(|| bad_uri("input_s3_uri", &cfg.input_s3_uri))?;
        let (out_bucket, out_prefix) =
            split_s3(&cfg.output_s3_uri).ok_or_else(|| bad_uri("output_s3_uri", &cfg.output_s3_uri))?;

        let mut results: Vec<BatchResult> = Vec::new();
        let mut sent: Vec<(String, String)> = Vec::new(); // (record id, request id)
        let mut lines = Vec::new();
        for r in requests {
            match self.batch_record(&r.invocation) {
                Ok(model_input) => {
                    let record_id = format!("REC{:08}", sent.len());
                    lines.push(json!({ "recordId": record_id, "modelInput": model_input }).to_string());
                    sent.push((record_id, r.id));
                }
                // a request that cannot go in a batch does not stop the others
                Err(e) => results.push(BatchResult {
                    id: r.id,
                    result: Err(e),
                }),
            }
        }
        if lines.is_empty() {
            return Ok(results);
        }

        let sdk = self.sdk_config().await;
        let s3 = {
            let mut b = aws_sdk_s3::config::Builder::from(sdk);
            // a server of our own (tests) has no bucket subdomains
            if self.endpoint_url.is_some() {
                b = b.force_path_style(true);
            }
            aws_sdk_s3::Client::from_conf(b.build())
        };
        let control = aws_sdk_bedrock::Client::new(sdk);

        let name = format!("a-graph-{}", uuid::Uuid::new_v4().simple());
        let input_key = format!("{in_prefix}{name}.jsonl");
        s3.put_object()
            .bucket(&in_bucket)
            .key(&input_key)
            .body(ByteStream::from((lines.join("\n") + "\n").into_bytes()))
            .send()
            .await
            .map_err(|e| self.sdk_error(e))?;

        let build = |e: aws_smithy_types::error::operation::BuildError| self.who.unsupported(vec![e.to_string()]);
        let input = ModelInvocationJobS3InputDataConfig::builder()
            .s3_uri(format!("s3://{in_bucket}/{input_key}"))
            .s3_input_format(S3InputFormat::Jsonl)
            .build()
            .map_err(build)?;
        let output = ModelInvocationJobS3OutputDataConfig::builder()
            .s3_uri(&cfg.output_s3_uri)
            .build()
            .map_err(build)?;
        let job = control
            .create_model_invocation_job()
            .job_name(&name)
            .role_arn(&cfg.role_arn)
            .model_id(&self.model)
            .model_invocation_type(ModelInvocationType::Converse)
            .input_data_config(ModelInvocationJobInputDataConfig::S3InputDataConfig(input))
            .output_data_config(ModelInvocationJobOutputDataConfig::S3OutputDataConfig(output))
            .send()
            .await
            .map_err(|e| self.sdk_error(e))?;
        let arn = job.job_arn().to_string();
        let job_id = arn.rsplit('/').next().unwrap_or(&arn).to_string();
        let _ = events.send(BatchEvent::Submitted {
            batch_id: job_id.clone(),
            requests: sent.len(),
        });

        // polled until a terminal state: there is no maximum wait
        let (status, message) = loop {
            let got = control
                .get_model_invocation_job()
                .job_identifier(&arn)
                .send()
                .await
                .map_err(|e| self.sdk_error(e))?;
            let status = got.status().cloned().unwrap_or(ModelInvocationJobStatus::Submitted);
            let _ = events.send(BatchEvent::Status {
                batch_id: job_id.clone(),
                status: status.as_str().to_string(),
            });
            if matches!(
                status,
                ModelInvocationJobStatus::Completed
                    | ModelInvocationJobStatus::PartiallyCompleted
                    | ModelInvocationJobStatus::Failed
                    | ModelInvocationJobStatus::Stopped
                    | ModelInvocationJobStatus::Expired
            ) {
                break (status, got.message().map(str::to_string));
            }
            tokio::time::sleep(self.poll_interval).await;
        };
        let outcome = match status {
            ModelInvocationJobStatus::Completed | ModelInvocationJobStatus::PartiallyCompleted => {
                BatchOutcome::Completed
            }
            ModelInvocationJobStatus::Expired => BatchOutcome::Expired,
            _ => BatchOutcome::Failed,
        };
        let _ = events.send(BatchEvent::Finished {
            batch_id: job_id.clone(),
            outcome,
        });

        // every `.jsonl.out` file of the job folder
        let folder = format!("{out_prefix}{job_id}/");
        let listed = s3
            .list_objects_v2()
            .bucket(&out_bucket)
            .prefix(&folder)
            .send()
            .await
            .map_err(|e| self.sdk_error(e))?;
        let keys: Vec<String> = listed
            .contents()
            .iter()
            .filter_map(|o| o.key())
            .filter(|k| k.ends_with(".jsonl.out"))
            .map(str::to_string)
            .collect();
        if keys.is_empty() {
            let why = message.unwrap_or_else(|| "no result file".into());
            return Err(self
                .who
                .provider_error(None, format!("batch {}: {why}", status.as_str())));
        }

        let mut by_record: HashMap<String, Value> = HashMap::new();
        for key in keys {
            let obj = s3
                .get_object()
                .bucket(&out_bucket)
                .key(&key)
                .send()
                .await
                .map_err(|e| self.sdk_error(e))?;
            let bytes = obj
                .body
                .collect()
                .await
                .map_err(|e| self.who.network_text(e.to_string()))?
                .into_bytes();
            for line in String::from_utf8_lossy(&bytes).lines().filter(|l| !l.trim().is_empty()) {
                let v: Value = serde_json::from_str(line)
                    .map_err(|e| self.who.invalid(format!("a result line is not JSON: {e}")))?;
                if let Some(id) = v.get("recordId").and_then(Value::as_str) {
                    by_record.insert(id.to_string(), v);
                }
            }
        }

        for (record_id, id) in sent {
            let result = match by_record.remove(&record_id) {
                None => Err(self.who.invalid(format!("the batch has no result for request `{id}`"))),
                Some(line) => self.batch_result(&line),
            };
            results.push(BatchResult { id, result });
        }
        Ok(results)
    }

    /// `StopModelInvocationJob`, with the id of the job: the API takes it with or without the rest of the ARN.
    pub(super) async fn cancel(&self, job_id: &str) -> Result<(), ModelError> {
        aws_sdk_bedrock::Client::new(self.sdk_config().await)
            .stop_model_invocation_job()
            .job_identifier(job_id)
            .send()
            .await
            .map(drop)
            .map_err(|e| self.sdk_error(e))
    }

    fn batch_result(&self, line: &Value) -> Result<ModelResponse, ModelError> {
        if let Some(err) = line.get("error").filter(|e| !e.is_null()) {
            let msg = err
                .get("errorMessage")
                .and_then(Value::as_str)
                .unwrap_or("the request failed");
            let status = err.get("errorCode").and_then(Value::as_u64).map(|c| c as u16);
            return Err(self.who.provider_error(status, format!("batch request failed: {msg}")));
        }
        let out = line
            .get("modelOutput")
            .ok_or_else(|| self.who.invalid("a result with neither `modelOutput` nor `error`"))?;
        // the time of one request inside a batch is not measurable: 0
        self.parse_converse_json(out, 0.0)
    }

    /// The `modelInput` of a record: a Converse request body. Tools, a schema, produced media and
    /// tool rounds do not go in a batch.
    fn batch_record(&self, call: &ModelInvocation) -> Result<Value, ModelError> {
        let mut problems = Vec::new();
        if !call.tools.is_empty() || !call.tool_rounds.is_empty() {
            problems.push("a batch on Bedrock cannot use tools".to_string());
        }
        if call.schema.is_some() {
            problems.push("a batch on Bedrock cannot use an output schema".to_string());
        }
        if !call.output_media.is_empty() {
            problems.push("media output is text only for now".to_string());
        }
        if !problems.is_empty() {
            return Err(self.who.unsupported(problems));
        }

        let mut body = Map::new();
        if !call.system.is_empty() {
            body.insert("system".into(), json!([{ "text": call.system }]));
        }
        let mut messages: Vec<Value> = call
            .history
            .iter()
            .map(|e| json!({ "role": if e.role == Role::User { "user" } else { "assistant" }, "content": [{ "text": e.content }] }))
            .collect();
        let mut content = Vec::new();
        for m in &call.media {
            match m {
                MediaInput::Markdown(md) => content.push(json!({ "text": md })),
                MediaInput::Image { format, data } => {
                    let f = super::convert::image_format(format).ok_or_else(|| {
                        self.who.unsupported(vec![format!(
                            "image format `{format}` is not accepted (png, jpeg, gif, webp)"
                        )])
                    })?;
                    content.push(json!({ "image": { "format": f.as_str(), "source": { "bytes": data } } }));
                }
            }
        }
        if !call.user.is_empty() {
            content.push(json!({ "text": call.user }));
        }
        messages.push(json!({ "role": "user", "content": content }));
        body.insert("messages".into(), Value::Array(messages));

        let mut inference = Map::new();
        if let Some(n) = call.max_tokens {
            inference.insert("maxTokens".into(), json!(n));
        }
        if let Some(t) = call.temperature {
            inference.insert("temperature".into(), json!(t));
        }
        if let Some(p) = call.top_p {
            inference.insert("topP".into(), json!(p));
        }
        if !inference.is_empty() {
            body.insert("inferenceConfig".into(), Value::Object(inference));
        }
        // reasoning, the way `converse` sends it
        match call.reasoning {
            None => {}
            Some(Reasoning::Off) => {
                body.insert(
                    "additionalModelRequestFields".into(),
                    json!({ "thinking": { "type": "disabled" } }),
                );
            }
            Some(level) => {
                let effort = match level {
                    Reasoning::Minimal | Reasoning::Low => "low",
                    Reasoning::Medium => "medium",
                    Reasoning::High => "high",
                    _ => "max",
                };
                body.insert("outputConfig".into(), json!({ "effort": effort }));
                body.insert(
                    "additionalModelRequestFields".into(),
                    json!({ "thinking": { "type": "adaptive", "display": "summarized" } }),
                );
            }
        }
        Ok(Value::Object(body))
    }

    /// The same mapping as `parse_converse`, from the JSON of a batch result.
    fn parse_converse_json(&self, v: &Value, execution_time: f64) -> Result<ModelResponse, ModelError> {
        let blocks = v
            .pointer("/output/message/content")
            .and_then(Value::as_array)
            .ok_or_else(|| self.who.invalid("no message in the result"))?;
        let mut text = String::new();
        let mut reasoning = Vec::new();
        let mut tools_call = Vec::new();
        for b in blocks {
            if let Some(t) = b.get("text").and_then(Value::as_str) {
                text.push_str(t);
            } else if let Some(r) = b.pointer("/reasoningContent/reasoningText") {
                reasoning.push(json!({ "text": r.get("text"), "signature": r.get("signature") }));
            } else if let Some(r) = b.pointer("/reasoningContent/redactedContent").and_then(Value::as_str) {
                reasoning.push(json!({ "redacted": r }));
            } else if let Some(t) = b.get("toolUse") {
                tools_call.push(ToolCall {
                    call_id: t
                        .get("toolUseId")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    function_name: t.get("name").and_then(Value::as_str).unwrap_or_default().to_string(),
                    arguments: t
                        .get("input")
                        .cloned()
                        .filter(Value::is_object)
                        .ok_or_else(|| self.who.invalid("tool call arguments are not a JSON object"))?,
                    opaque: None,
                });
            }
        }
        let stop = v.get("stopReason").and_then(Value::as_str).unwrap_or_default();
        let mut stop_reason = match stop {
            "end_turn" | "stop_sequence" => StopReason::End,
            "tool_use" => StopReason::ToolCall,
            "max_tokens" => StopReason::MaxTokens,
            "model_context_window_exceeded" => StopReason::ContextExceeded,
            "guardrail_intervened" | "content_filtered" => StopReason::Refused,
            _ => StopReason::Other,
        };
        if !tools_call.is_empty() {
            stop_reason = StopReason::ToolCall;
        }
        let n = |p: &str| v.pointer(p).and_then(Value::as_u64).unwrap_or(0);
        let usage = Usage {
            input_tokens: n("/usage/inputTokens"),
            cache_read_tokens: n("/usage/cacheReadInputTokens"),
            cache_write_tokens: n("/usage/cacheWriteInputTokens"),
            output_tokens: n("/usage/outputTokens"),
            reasoning_tokens: 0,
        };
        Ok(ModelResponse {
            message: (!text.is_empty()).then_some(text),
            structured: None,
            media: Vec::new(),
            tools_call,
            reasoning,
            stop_reason,
            stop_detail: Some(stop.to_string()),
            info: ResponseInfo {
                provider: self.who.provider.into(),
                model: self.model.clone(),
                execution_time,
                usage,
            },
        })
    }
}
