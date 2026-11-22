use crate::common::run_batch::*;
use a_graph::models::{
    BatchEvents, BatchRequest, BatchResult, BatchReturn, Llm, ModelError, ModelInvocation, ModelRequirements,
    ModelResponse, ResponseInfo, StopReason, Usage,
};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

fn answer(text: &str) -> ModelResponse {
    ModelResponse {
        message: Some(text.into()),
        structured: None,
        media: vec![],
        tools_call: vec![],
        reasoning: vec![],
        logprobs: vec![vec![]],
        stop_reason: StopReason::End,
        stop_detail: None,
        info: ResponseInfo {
            provider: "test".into(),
            model: "m".into(),
            execution_time: 0.0,
            usage: Usage::default(),
            corrections: None,
        },
    }
}

/// A model that answers every call and every request of a batch with its text, and never with logprobs. It
/// writes down whether each request of the batch asked them.
struct NoLogprobs {
    asked: Arc<Mutex<Vec<Option<u32>>>>,
}

#[async_trait::async_trait]
impl Llm for NoLogprobs {
    fn check(&self, _: &ModelRequirements) -> Result<(), ModelError> {
        Ok(())
    }

    async fn invoke(&self, call: &ModelInvocation) -> Result<ModelResponse, ModelError> {
        Ok(answer(&call.user))
    }

    async fn invoke_batch(&self, requests: Vec<BatchRequest>, _: BatchEvents) -> BatchReturn {
        Ok(requests
            .into_iter()
            .map(|r| {
                self.asked.lock().unwrap().push(r.invocation.logprobs);
                BatchResult {
                    id: r.id,
                    result: Ok(answer(&r.invocation.user)),
                }
            })
            .collect())
    }
}

/// The requests of a batch ask the logprobs as a single call does; an answer of the batch with a text and no
/// logprobs stops the run and names its node.
#[tokio::test]
async fn logprobs_missing() {
    // SAFETY: the value is never used to call anything.
    unsafe { std::env::set_var("OPENAI_API_KEY", "o") };
    let mut p = build(&yml_in(here(), "logprobs_missing")).unwrap_or_else(|e| panic!("{e}"));
    let asked = Arc::new(Mutex::new(Vec::new()));
    p.models.insert("m", Box::new(NoLogprobs { asked: asked.clone() }));
    let e = Arc::new(p).run("hi").await.unwrap_err();
    assert_eq!(*asked.lock().unwrap(), vec![Some(0), Some(0)]);
    assert_eq!(
        (e.rule.code(), e.path.to_string()),
        ("model_failed", "node(d)".to_string())
    );
    assert!(e.message.contains("logprobs"), "{}", e.message);
}
