//! The batch jobs a run lets go of before they ended: the run stops (`run_timeout`, the service shutting down), a
//! branch fails beside the group, or the poll of the job fails. The job would go on at the provider, billed, and its
//! results would never be read: the provider is asked to cancel it, and the end of the program waits for that.

use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::task::{JoinError, JoinHandle};

use super::Program;
use crate::models::cannot_cancel;
use crate::models::{BatchEvent, BatchReturn, Llm};

/// The longest a cancel is waited for, with the creation of the job when it was on its way: a provider that does not
/// answer must not hold the end of the run. It fits in the 5 seconds the service gives a run it stopped to close
/// (`CLOSE_SECONDS`), with time left for the log, which is closed after the cancels.
const CANCEL_SECONDS: u64 = 3;

/// How long the end of the program waits for the branches a run aborted to let go of it.
const BRANCHES_WAIT: Duration = Duration::from_secs(1);

/// The cancels sent and not yet answered.
pub(super) type Cancels = Mutex<Vec<JoinHandle<()>>>;

/// The call of `invoke_batch` of one chunk, a task of its own (`job`), as long as its jobs may run at the provider.
/// The provider tells `Creating` just before the request that creates a job, `Submitted` as soon as it has its id,
/// and `Finished` when the job ended (`crate::models::batch`). When this guard is dropped before the call ended, it
/// closes the channel first: from then on the provider cannot tell anything, so it does not create a job, and it
/// cancels by itself a job whose id comes later. Then the guard reads all that was told before: a job submitted and
/// not finished is cancelled here and the task is stopped; a job whose creation is on its way is left to the task,
/// which is waited for; with nothing told, the task is stopped and no job exists. When the call ended with a job
/// submitted and not finished (its poll failed), that job is cancelled too. Dropping is synchronous, so what is
/// waited for is a task of its own, kept in `cancels` for the end of the program to wait for.
pub(super) struct OpenBatch<'a> {
    heard: UnboundedReceiver<BatchEvent>,
    job: Option<JoinHandle<BatchReturn>>,
    model: Arc<dyn Llm>,
    cancels: &'a Cancels,
}

impl<'a> OpenBatch<'a> {
    pub(super) fn new(
        heard: UnboundedReceiver<BatchEvent>,
        job: JoinHandle<BatchReturn>,
        model: Arc<dyn Llm>,
        cancels: &'a Cancels,
    ) -> Self {
        OpenBatch {
            heard,
            job: Some(job),
            model,
            cancels,
        }
    }

    /// The outcome of the call. Dropped while it waits, the guard is dropped with the call still going on.
    pub(super) async fn outcome(&mut self) -> Result<BatchReturn, JoinError> {
        let job = self.job.as_mut().expect("the outcome is read once");
        let outcome = job.await;
        self.job = None;
        outcome
    }
}

impl Drop for OpenBatch<'_> {
    fn drop(&mut self) {
        self.heard.close();
        let (mut open, mut creating): (Vec<String>, bool) = (Vec::new(), false);
        // on a closed channel `Empty` means a message is still on its way from another thread: it is waited for,
        // and `Disconnected` comes when every message told before the close was read
        loop {
            match self.heard.try_recv() {
                Ok(BatchEvent::Creating { .. }) => creating = true,
                Ok(BatchEvent::Submitted { batch_id, .. }) => {
                    creating = false;
                    open.push(batch_id);
                }
                Ok(BatchEvent::Finished { batch_id, .. }) => open.retain(|id| *id != batch_id),
                Ok(BatchEvent::Status { .. }) => {}
                Err(TryRecvError::Empty) => std::thread::yield_now(),
                Err(TryRecvError::Disconnected) => break,
            }
        }
        // without a runtime (the process is ending) nothing can be sent or waited for any more
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            if let Some(job) = self.job.take() {
                job.abort();
            }
            return;
        };
        let mut cancels = self.cancels.lock().unwrap_or_else(PoisonError::into_inner);
        match self.job.take() {
            // the call ended, with an error when a job is still open: its poll failed
            None => {}
            Some(job) if creating => {
                // the provider ends the creation, and cancels the job as soon as it has its id
                cancels.push(runtime.spawn(async move {
                    let stop = job.abort_handle();
                    if tokio::time::timeout(Duration::from_secs(CANCEL_SECONDS), job)
                        .await
                        .is_err()
                    {
                        stop.abort();
                        eprintln!(
                            "a-graph: a batch job was being created when the run stopped, and its id did not come \
                             within {CANCEL_SECONDS} seconds: it may be left running at the provider"
                        );
                    }
                }));
            }
            // nobody waits for the task any more: its poll stops here
            Some(job) => job.abort(),
        }
        for id in open {
            let model = self.model.clone();
            cancels.push(runtime.spawn(async move {
                match tokio::time::timeout(Duration::from_secs(CANCEL_SECONDS), model.cancel_batch(&id)).await {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => cannot_cancel(&id, &e.to_string()),
                    Err(_) => cannot_cancel(&id, &format!("no answer within {CANCEL_SECONDS} seconds")),
                }
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
