//! Batch calls: submit, poll and retrieve are hidden inside `Llm::invoke_batch`.

use tokio::sync::mpsc::UnboundedSender;

use super::error::ModelError;
use super::invocation::ModelInvocation;
use super::response::ModelResponse;

/// One request of a batch, matched to its result by `id`.
#[derive(Debug, Clone)]
pub struct BatchRequest {
    pub id: String,
    pub invocation: ModelInvocation,
}

#[derive(Debug)]
pub struct BatchResult {
    pub id: String,
    pub result: Result<ModelResponse, ModelError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchOutcome {
    Completed,
    Failed,
    Expired,
}

/// The client is always told the status; showing it is the job of the `verbose` reporter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchEvent {
    /// The request that creates the job is about to be sent: from here on a job may exist at the provider before
    /// its id is known. Before it, nothing was sent that makes a job.
    Creating {
        requests: usize,
    },
    Submitted {
        batch_id: String,
        requests: usize,
    },
    /// At EVERY poll. `status` is the provider's original value.
    Status {
        batch_id: String,
        status: String,
    },
    Finished {
        batch_id: String,
        outcome: BatchOutcome,
    },
}

pub type BatchEvents = UnboundedSender<BatchEvent>;

pub type BatchReturn = Result<Vec<BatchResult>, ModelError>;

/// How many times in a row the status of a batch job, or its results, may fail to come before the batch fails.
pub(crate) const BATCH_TRIES: u32 = 10;

/// An error that may not come again: the network, too many requests (429), a fault of the provider (5xx, the 529 of
/// Anthropic when it is overloaded).
fn transient(e: &ModelError) -> bool {
    match e {
        ModelError::Network { .. } => true,
        ModelError::Provider { status: Some(s), .. } => *s == 429 || (500..600).contains(s),
        _ => false,
    }
}

/// Reads the status of a job, or its results, as `read` does, and tries again after `interval` when the error may
/// not come again: a batch waits for hours, and one failed request must not end it while the job goes on. After
/// `BATCH_TRIES` failures in a row the last error is the outcome; any other error is at once. Never for the request
/// that creates a job: a request that arrived but whose answer was lost would create a second one.
pub(crate) async fn patiently<T, F, Fut>(interval: std::time::Duration, mut read: F) -> Result<T, ModelError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, ModelError>>,
{
    let mut failed = 0;
    loop {
        match read().await {
            Err(e) if transient(&e) && failed + 1 < BATCH_TRIES => {
                failed += 1;
                tokio::time::sleep(interval).await;
            }
            done => return done,
        }
    }
}
