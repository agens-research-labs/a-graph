use crate::common::run_batch::*;
use a_graph::error::{Error, Phase, Rule};
use a_graph::models::{
    BatchEvent, BatchEvents, BatchRequest, BatchReturn, Llm, ModelError, ModelInvocation, ModelRequirements,
    ModelResponse, ResponseInfo, StopReason, Usage,
};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A model whose batch spends `PREPARE` before the request that creates the job (as the upload of a file does), and
/// whose job never ends. It writes down the jobs it created and the cancels it was asked.
struct Slow {
    done: Arc<Mutex<Vec<String>>>,
}

const PREPARE: Duration = Duration::from_millis(500);

#[async_trait::async_trait]
impl Llm for Slow {
    fn check(&self, _: &ModelRequirements) -> Result<(), ModelError> {
        Ok(())
    }

    async fn invoke(&self, call: &ModelInvocation) -> Result<ModelResponse, ModelError> {
        Ok(ModelResponse {
            message: Some(call.user.clone()),
            structured: None,
            media: vec![],
            tools_call: vec![],
            reasoning: vec![],
            stop_reason: StopReason::End,
            stop_detail: None,
            info: ResponseInfo {
                provider: "slow".into(),
                model: "slow".into(),
                execution_time: 0.0,
                usage: Usage::default(),
            },
        })
    }

    async fn invoke_batch(&self, requests: Vec<BatchRequest>, events: BatchEvents) -> BatchReturn {
        tokio::time::sleep(PREPARE).await;
        let _ = events.send(BatchEvent::Creating {
            requests: requests.len(),
        });
        self.done.lock().unwrap().push("created b1".into());
        let _ = events.send(BatchEvent::Submitted {
            batch_id: "b1".into(),
            requests: requests.len(),
        });
        std::future::pending().await
    }

    async fn cancel_batch(&self, batch_id: &str) -> Result<(), ModelError> {
        self.done.lock().unwrap().push(format!("cancelled {batch_id}"));
        Ok(())
    }
}

/// A run stopped while its batch is still prepared, before the request that creates the job is sent: no job is
/// created, so there is nothing to cancel, and the end of the program does not wait for the preparation.
#[tokio::test]
async fn preparing_no_job() {
    let (mut p, _, _) = program_on(here(), "preparing_no_job", Jobs::default()).await;
    let done = Arc::new(Mutex::new(Vec::new()));
    Arc::get_mut(&mut p)
        .unwrap()
        .models
        .insert("m", Box::new(Slow { done: done.clone() }));
    let stop = async {
        tokio::time::sleep(Duration::from_millis(200)).await;
        Error::new(Phase::Execution, Path::root(), Rule::RunTimeout, "stopped")
    };
    let e = p.run_until("hi", stop).await.unwrap_err();
    assert_eq!(e.rule, Rule::RunTimeout);
    let closing = std::time::Instant::now();
    a_graph::run::close(p).await;
    assert!(closing.elapsed() < PREPARE, "{:?}", closing.elapsed());

    // past the time the preparation would have taken, the job was still never created
    tokio::time::sleep(PREPARE * 2).await;
    assert!(done.lock().unwrap().is_empty(), "{:?}", done.lock().unwrap());
}
