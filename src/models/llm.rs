//! The one trait every provider implements.

use async_trait::async_trait;

use super::batch::{BatchEvents, BatchRequest, BatchReturn};
use super::error::ModelError;
use super::invocation::ModelInvocation;
use super::requirements::ModelRequirements;
use super::response::ModelResponse;

#[async_trait]
pub trait Llm: Send + Sync {
    /// Does the provider support what the node asks? No network.
    fn check(&self, req: &ModelRequirements) -> Result<(), ModelError>;

    /// One call. One round only: the tool loop belongs to the node.
    async fn invoke(&self, call: &ModelInvocation) -> Result<ModelResponse, ModelError>;

    /// Submits the requests as one batch job, polls until a terminal state and returns the result
    /// of each id. One round: tool requests come back as they are. A failed or expired job is
    /// `ModelError::Provider`; the status is reported through `events`.
    async fn invoke_batch(&self, requests: Vec<BatchRequest>, events: BatchEvents) -> BatchReturn;

    /// Asks the provider to stop the batch job `batch_id`, the id `invoke_batch` sent in `Submitted`. The run
    /// calls it when it lets go of a job before the job ended. A provider without batch never sends `Submitted`,
    /// so it is never asked.
    async fn cancel_batch(&self, _batch_id: &str) -> Result<(), ModelError> {
        Ok(())
    }
}
