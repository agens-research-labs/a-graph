//! The batch jobs a run lets go of before they ended: the run stops (`run_timeout`, the service shutting down), a
//! branch fails beside the group, or the poll of the job fails. The job would go on at the provider, billed, and its
//! results would never be read: the provider is asked to cancel it, and the end of the program waits for that.

use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;

use super::Program;
use crate::models::{BatchEvent, Llm};

/// The longest a cancel is waited for: a provider that does not answer must not hold the end of the run.
const CANCEL_SECONDS: u64 = 10;

/// How long the end of the program waits for the branches a run aborted to let go of it.
const BRANCHES_WAIT: Duration = Duration::from_secs(1);

/// The cancels sent and not yet answered.
pub(super) type Cancels = Mutex<Vec<JoinHandle<()>>>;

/// The batch jobs of one call of `invoke_batch`, as long as they may run at the provider. The provider sends
/// `Submitted` as soon as it has the id of a job, before it waits for it, and `Finished` when the job ended. When
/// this guard is dropped, the events sent until then are all in the channel: a job submitted and not finished is
/// cancelled, whatever dropped the guard. Dropping is synchronous, so the cancel is a task of its own, kept in
/// `cancels` for the end of the program to wait for.
pub(super) struct OpenBatch<'a> {
    pub(super) heard: UnboundedReceiver<BatchEvent>,
    pub(super) model: Arc<dyn Llm>,
    pub(super) cancels: &'a Cancels,
}

impl Drop for OpenBatch<'_> {
    fn drop(&mut self) {
        let mut open: Vec<String> = Vec::new();
        while let Ok(event) = self.heard.try_recv() {
            match event {
                BatchEvent::Submitted { batch_id, .. } => open.push(batch_id),
                BatchEvent::Finished { batch_id, .. } => open.retain(|id| *id != batch_id),
                BatchEvent::Status { .. } => {}
            }
        }
        // without a runtime (the process is ending) there is nothing left to send the cancel with
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            return;
        };
        let mut cancels = self.cancels.lock().unwrap_or_else(PoisonError::into_inner);
        for id in open {
            let model = self.model.clone();
            cancels.push(runtime.spawn(async move {
                let why = match tokio::time::timeout(Duration::from_secs(CANCEL_SECONDS), model.cancel_batch(&id)).await
                {
                    Ok(Ok(())) => return,
                    Ok(Err(e)) => e.to_string(),
                    Err(_) => format!("no answer within {CANCEL_SECONDS} seconds"),
                };
                eprintln!("a-graph: cannot cancel the batch job {id} at the provider: {why}");
            }));
        }
    }
}

impl Program {
    /// Waits for the cancels of the batch jobs the run let go of (`OpenBatch`), each for `CANCEL_SECONDS` at most.
    pub async fn cancels_sent(&self) {
        let pending = std::mem::take(&mut *self.cancels.lock().unwrap_or_else(PoisonError::into_inner));
        for cancel in pending {
            let _ = cancel.await;
        }
    }
}

/// Ends a program after its run. The branches a stopped or failed run aborted let go of it a moment later, and
/// only then are their batch jobs cancelled: they are waited for, `BRANCHES_WAIT` at most, then the cancels, then
/// the log delivers what is still queued. A branch that still holds the program after `BRANCHES_WAIT` keeps the
/// log open; the cancels sent until then are waited for all the same.
pub async fn close(program: Arc<Program>) {
    let mut program = program;
    let until = tokio::time::Instant::now() + BRANCHES_WAIT;
    loop {
        match Arc::try_unwrap(program) {
            Ok(p) => {
                p.cancels_sent().await;
                p.logger.close().await;
                return;
            }
            Err(still) if tokio::time::Instant::now() < until => {
                program = still;
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(still) => {
                still.cancels_sent().await;
                return;
            }
        }
    }
}
