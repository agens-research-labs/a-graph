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
