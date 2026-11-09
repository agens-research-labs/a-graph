use crate::common::http::{Req, Resp, serve};
use a_graph::models::*;
use a_graph::resources::{Generate, MediaKind};
use a_graph::{Path, Rule};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

fn model(creds: BedrockCredentials) -> BedrockModel {
    BedrockModel {
        common: Common {
            id: "strong".into(),
            model: "anthropic.claude-sonnet-4-5".into(),
            input: vec![Modality::Text],
            output: vec![Modality::Text],
        },
        region: "eu-west-1".into(),
        credentials: creds,
        batch: None,
    }
}

fn keys() -> BedrockCredentials {
    // SAFETY: variables only these tests use, set to the same value every time.
    unsafe {
        std::env::set_var("A_GRAPH_TEST_AWS_AK", "AKIATEST");
        std::env::set_var("A_GRAPH_TEST_AWS_SK", "secret-test-key");
    }
    BedrockCredentials::Keys {
        access_key: "A_GRAPH_TEST_AWS_AK".into(),
        secret_key: "A_GRAPH_TEST_AWS_SK".into(),
        session_token: None,
    }
}

fn client(base: &str) -> Bedrock {
    Bedrock::new(&model(keys()), &Path::root().key("models").idx(0))
        .unwrap()
        .with_endpoint_url(base)
}

fn call(user: &str) -> ModelInvocation {
    ModelInvocation {
        system: String::new(),
        history: vec![],
        user: user.into(),
        media: vec![],
        tools: vec![],
        output_media: vec![],
        schema: None,
        temperature: None,
        top_p: None,
        max_tokens: None,
        reasoning: None,
        tool_rounds: vec![],
    }
}

fn converse(content: Value, stop: &str) -> String {
    json!({ "output": { "message": { "role": "assistant", "content": content } }, "stopReason": stop, "metrics": { "latencyMs": 50 },
            "usage": { "inputTokens": 30, "outputTokens": 9, "totalTokens": 39, "cacheReadInputTokens": 5, "cacheWriteInputTokens": 2 } })
    .to_string()
}

fn text_answer(text: &str) -> String {
    converse(json!([{ "text": text }]), "end_turn")
}

async fn one_call(call: ModelInvocation, answer: Resp) -> (Result<ModelResponse, ModelError>, Req) {
    let answer = Arc::new(Mutex::new(Some(answer)));
    let (base, seen) = serve(move |_| answer.lock().unwrap().take().unwrap_or(Resp::ok("{}"))).await;
    let result = client(&base).invoke(&call).await;
    let req = seen.lock().unwrap()[0].clone();
    (result, req)
}

fn body(req: &Req) -> Value {
    serde_json::from_str(&req.body).unwrap()
}

trait WithErrorType {
    fn with_error_type(self, kind: &str) -> Self;
}

impl WithErrorType for Resp {
    // the AWS SDK reads the error type from this header
    fn with_error_type(self, kind: &str) -> Self {
        Resp::status(
            self.status,
            json!({ "__type": kind, "message": serde_json::from_str::<Value>(&self.body).unwrap()["message"] })
                .to_string(),
        )
    }
}

// --- batch ---------------------------------------------------------------------------------

fn batch_model() -> BedrockModel {
    BedrockModel {
        batch: Some(BedrockBatch {
            input_s3_uri: "s3://my-in/jobs".into(),
            output_s3_uri: "s3://my-out/results/".into(),
            role_arn: "arn:aws:iam::123456789012:role/batch".into(),
        }),
        ..model(keys())
    }
}

fn batch_client(base: &str) -> Bedrock {
    Bedrock::new(&batch_model(), &Path::root())
        .unwrap()
        .with_endpoint_url(base)
        .with_poll_interval(std::time::Duration::from_millis(10))
}

const JOB_ARN: &str = "arn:aws:bedrock:eu-west-1:123456789012:model-invocation-job/job42";

fn job(status: &str) -> String {
    json!({
        "jobArn": JOB_ARN, "roleArn": "arn:aws:iam::123456789012:role/batch", "modelId": "anthropic.claude-sonnet-4-5", "status": status, "message": "the job could not start",
        "submitTime": "2025-10-12T10:00:00Z", "inputDataConfig": { "s3InputDataConfig": { "s3Uri": "s3://my-in/jobs/x.jsonl" } }, "outputDataConfig": { "s3OutputDataConfig": { "s3Uri": "s3://my-out/results/" } }
    })
    .to_string()
}

fn batch_server(
    polls_before_done: usize,
    final_status: &'static str,
    with_output: bool,
) -> (impl Fn(&Req) -> Resp + Send + Sync + 'static, Arc<Mutex<usize>>) {
    let polls = Arc::new(Mutex::new(0usize));
    let seen = polls.clone();
    let record = |rec: &str, out: Value| json!({ "recordId": rec, "modelInput": {}, "modelOutput": out }).to_string();
    let answer = |text: &str| serde_json::from_str::<Value>(&text_answer(text)).unwrap();
    let output = [
        record("REC00000001", answer("second")),
        record("REC00000000", answer("first")),
        json!({ "recordId": "REC00000002", "modelInput": {}, "error": { "errorCode": 400, "errorMessage": "bad request" } }).to_string(),
    ]
    .join("\n");
    let listing = r#"<?xml version="1.0" encoding="UTF-8"?><ListBucketResult><Name>my-out</Name><Prefix>results/job42/</Prefix><KeyCount>2</KeyCount><MaxKeys>1000</MaxKeys><IsTruncated>false</IsTruncated><Contents><Key>results/job42/manifest.json.out</Key><Size>10</Size></Contents><Contents><Key>results/job42/x.jsonl.out</Key><Size>100</Size></Contents></ListBucketResult>"#;
    let handler = move |req: &Req| {
        let path = req.path.split('?').next().unwrap_or("").trim_end_matches('/');
        match (req.method.as_str(), path) {
            ("PUT", p) if p.starts_with("/my-in/jobs/a-graph-") && p.ends_with(".jsonl") => Resp::ok(""),
            ("POST", "/model-invocation-job") => Resp::ok(json!({ "jobArn": JOB_ARN }).to_string()),
            ("GET", p) if p.starts_with("/model-invocation-job/") => {
                let mut n = polls.lock().unwrap();
                *n += 1;
                Resp::ok(job(if *n > polls_before_done {
                    final_status
                } else {
                    "InProgress"
                }))
            }
            ("GET", "/my-out") if with_output => Resp::ok(listing),
            ("GET", "/my-out") => Resp::ok(
                r#"<?xml version="1.0"?><ListBucketResult><Name>my-out</Name><KeyCount>0</KeyCount><IsTruncated>false</IsTruncated></ListBucketResult>"#,
            ),
            ("GET", "/my-out/results/job42/x.jsonl.out") => Resp::ok(output.clone()),
            _ => Resp::status(404, "{}"),
        }
    };
    (handler, seen)
}

#[path = "plain_call/plain_call.rs"]
mod plain_call;

#[path = "reasoning/reasoning.rs"]
mod reasoning;

#[path = "tool_round/tool_round.rs"]
mod tool_round;

#[path = "structured_stops/structured_stops.rs"]
mod structured_stops;

#[path = "error_messages/error_messages.rs"]
mod error_messages;

#[path = "keys_from_env/keys_from_env.rs"]
mod keys_from_env;

#[path = "check_limits/check_limits.rs"]
mod check_limits;

#[path = "batch_s3/batch_s3.rs"]
mod batch_s3;

#[path = "failed_job/failed_job.rs"]
mod failed_job;

#[path = "no_batch_block/no_batch_block.rs"]
mod no_batch_block;

#[path = "batch_cancel/batch_cancel.rs"]
mod batch_cancel;

#[path = "batch_let_go/batch_let_go.rs"]
mod batch_let_go;

#[path = "structured_refusal/structured_refusal.rs"]
mod structured_refusal;

#[path = "cut_tool_call/cut_tool_call.rs"]
mod cut_tool_call;
