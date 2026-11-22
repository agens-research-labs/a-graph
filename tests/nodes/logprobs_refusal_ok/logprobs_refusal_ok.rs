use crate::common::run_node::*;
use a_graph::models::{
    BatchEvents, BatchRequest, BatchReturn, Llm, ModelError, ModelInvocation, ModelRequirements, ModelResponse,
    ResponseInfo, StopReason, Usage,
};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A model that refuses: its text is the refusal, with no logprobs, as OpenAI gives it.
struct Refuses;

#[async_trait::async_trait]
impl Llm for Refuses {
    fn check(&self, _: &ModelRequirements) -> Result<(), ModelError> {
        Ok(())
    }

    async fn invoke(&self, _: &ModelInvocation) -> Result<ModelResponse, ModelError> {
        Ok(ModelResponse {
            message: Some("I cannot help with that".into()),
            structured: None,
            media: vec![],
            tools_call: vec![],
            reasoning: vec![],
            logprobs: vec![vec![]],
            stop_reason: StopReason::Refused,
            stop_detail: None,
            info: ResponseInfo {
                provider: "test".into(),
                model: "m".into(),
                execution_time: 0.0,
                usage: Usage::default(),
                corrections: None,
            },
        })
    }

    async fn invoke_batch(&self, _: Vec<BatchRequest>, _: BatchEvents) -> BatchReturn {
        unreachable!("no batch here")
    }
}

/// A refusal is not the answer the model wrote and has no logprobs: it is given as it is, not
/// taken for logprobs that are missing.
#[tokio::test]
async fn logprobs_refusal_ok() {
    let (mut p, _) = program(here(), "logprobs_refusal_ok", Resp::ok(chat("unused", None))).await;
    p.models.insert("m", Box::new(Refuses));
    let out = p.run_node(entity(&p), user("q")).await.unwrap();
    assert_eq!(
        out.assistant.message,
        Some(Content::Text("I cannot help with that".into()))
    );
    assert_eq!(out.assistant.logprobs, vec![Vec::new()]);
}
