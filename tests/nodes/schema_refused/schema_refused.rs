use crate::common::run_node::*;
use a_graph::models::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A model that refuses, as a provider that reports refusals tells it.
struct Refuses;

#[async_trait::async_trait]
impl Llm for Refuses {
    fn check(&self, _: &ModelRequirements) -> Result<(), ModelError> {
        Ok(())
    }

    async fn invoke(&self, _: &ModelInvocation) -> Result<ModelResponse, ModelError> {
        Ok(ModelResponse {
            message: Some("I can't help with the diabetes of the patient".into()),
            structured: None,
            media: vec![],
            tools_call: vec![],
            reasoning: vec![],
            logprobs: vec![],
            stop_reason: StopReason::Refused,
            stop_detail: Some("refusal".into()),
            info: ResponseInfo {
                provider: "fake".into(),
                model: "m".into(),
                execution_time: 0.0,
                usage: Usage::default(),
            },
        })
    }

    async fn invoke_batch(&self, _: Vec<BatchRequest>, _: BatchEvents) -> BatchReturn {
        Ok(vec![])
    }
}

/// With `schema`, a refusal stops the node (`model_refused`); what the model wrote is the detail, never the
/// message.
#[tokio::test]
async fn schema_refused() {
    let (mut p, _) = program(here(), "schema_refused", Resp::ok(chat("{}", None))).await;
    p.models.insert("m", Box::new(Refuses));
    let e = p.run_node(entity(&p), user("q")).await.unwrap_err();
    assert_eq!(e.rule.code(), "model_refused");
    assert!(!e.message.contains("diabetes"), "{}", e.message);
    assert!(e.detail.as_deref().is_some_and(|d| d.contains("diabetes")), "{e:?}");
}
