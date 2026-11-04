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
    /// its id is known. Before it, nothing was sent that makes a job; when it cannot be told, no job is made.
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

/// Tells `Creating`, just before the request that creates the job. `false` when nobody waits for the batch any more
/// (the run let go of it): then the job must not be created. The run closes the channel before it looks at what was
/// told, so a job is created only if the run will know it is on its way.
pub(crate) fn may_create(events: &BatchEvents, requests: usize) -> bool {
    events.send(BatchEvent::Creating { requests }).is_ok()
}

/// Tells `Submitted`, as soon as the id of the job is known. When nobody waits for the batch any more, the run let go
/// of it while the job was created: only the provider knows the id, so it cancels the job here (`cancel`), and
/// `false` is returned.
pub(crate) async fn submitted(
    events: &BatchEvents,
    batch_id: &str,
    requests: usize,
    cancel: impl std::future::Future<Output = Result<(), ModelError>>,
) -> bool {
    let told = BatchEvent::Submitted {
        batch_id: batch_id.to_string(),
        requests,
    };
    if events.send(told).is_ok() {
        return true;
    }
    if let Err(e) = cancel.await {
        cannot_cancel(batch_id, &e.to_string());
    }
    false
}

/// What is written when a job left behind cannot be cancelled: it goes on at the provider, and this is its id.
pub(crate) fn cannot_cancel(batch_id: &str, why: &str) {
    eprintln!("a-graph: cannot cancel the batch job {batch_id} at the provider: {why}");
}
