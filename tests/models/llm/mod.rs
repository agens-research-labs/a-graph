use a_graph::models::*;
use async_trait::async_trait;

/// A model that answers without any network: enough to exercise the contract.
struct Fake {
    id: &'static str,
}

impl Fake {
    fn response(&self, text: String) -> ModelResponse {
        ModelResponse {
            message: Some(text),
            structured: None,
            media: vec![],
            tools_call: vec![],
            reasoning: vec![],
            logprobs: vec![],
            stop_reason: StopReason::End,
            stop_detail: Some("stop".into()),
            info: ResponseInfo {
                provider: "fake".into(),
                model: self.id.into(),
                execution_time: 0.5,
                usage: Usage {
                    input_tokens: 10,
                    output_tokens: 5,
                    ..Usage::default()
                },
                corrections: None,
            },
        }
    }
}

#[async_trait]
impl Llm for Fake {
    fn check(&self, req: &ModelRequirements) -> Result<(), ModelError> {
        let mut problems = Vec::new();
        if req.batch {
            problems.push("batch is not supported".to_string());
        }
        if req.uses_tools && req.schema.is_some() {
            problems.push("tools and schema together".to_string());
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(ModelError::Unsupported {
                model: self.id.into(),
                provider: "fake".into(),
                problems,
            })
        }
    }

    async fn invoke(&self, call: &ModelInvocation) -> Result<ModelResponse, ModelError> {
        Ok(self.response(format!("{}|{}", call.system, call.user)))
    }

    async fn invoke_batch(&self, requests: Vec<BatchRequest>, events: BatchEvents) -> BatchReturn {
        let batch_id = "b1".to_string();
        let _ = events.send(BatchEvent::Submitted {
            batch_id: batch_id.clone(),
            requests: requests.len(),
        });
        let _ = events.send(BatchEvent::Status {
            batch_id: batch_id.clone(),
            status: "in_progress".into(),
        });
        let _ = events.send(BatchEvent::Status {
            batch_id: batch_id.clone(),
            status: "ended".into(),
        });
        let _ = events.send(BatchEvent::Finished {
            batch_id,
            outcome: BatchOutcome::Completed,
        });
        // results come back in any order: the id is what matches them
        Ok(requests
            .into_iter()
            .rev()
            .map(|r| BatchResult {
                id: r.id,
                result: Ok(self.response(r.invocation.user)),
            })
            .collect())
    }
}

fn invocation(system: &str, user: &str) -> ModelInvocation {
    ModelInvocation {
        system: system.into(),
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
        logprobs: None,
        tool_rounds: vec![],
        corrections: vec![],
    }
}

#[path = "call_by_id/call_by_id.rs"]
mod call_by_id;

#[path = "check_names_all/check_names_all.rs"]
mod check_names_all;

#[path = "usage_adds/usage_adds.rs"]
mod usage_adds;

#[path = "batch_status/batch_status.rs"]
mod batch_status;

#[path = "error_names_model/error_names_model.rs"]
mod error_names_model;

#[path = "cut_says_why/cut_says_why.rs"]
mod cut_says_why;

#[path = "structured_check/structured_check.rs"]
mod structured_check;

#[path = "manager_serves/manager_serves.rs"]
mod manager_serves;
