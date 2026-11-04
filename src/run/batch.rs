//! The batch jobs a run lets go of before they ended: the run stops (`run_timeout`, the service shutting down), a
//! branch fails beside the group, or the poll of the job fails. The job would go on at the provider, billed, and its
//! results would never be read: the provider is asked to cancel it, and the end of the program waits for that.

use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::{AbortHandle, JoinHandle};

use super::Program;
use crate::models::{BatchEvent, Llm};

/// The longest a cancel is waited for, with the creation of the job when it was still going on: a provider that does
/// not answer must not hold the end of the run. It fits in the 5 seconds the service gives a run it stopped to close
/// (`CLOSE_SECONDS`), with time left for the log, which is closed after the cancels.
const CANCEL_SECONDS: u64 = 3;

/// How long the end of the program waits for the branches a run aborted to let go of it.
const BRANCHES_WAIT: Duration = Duration::from_secs(1);

/// The cancels sent and not yet answered.
pub(super) type Cancels = Mutex<Vec<JoinHandle<()>>>;

/// The call of `invoke_batch` of one chunk, a task of its own (`job`), as long as its jobs may run at the provider.
/// The provider sends `Creating` just before the request that creates a job, `Submitted` as soon as it has its id,
/// and `Finished` when the job ended. When this guard is dropped, the events sent until then are all in the channel,
/// and the task is stopped: a job submitted and not finished is cancelled, whatever dropped the guard; before
/// `Creating` nothing was sent, and nothing is left. Between `Creating` and `Submitted` the request that creates the
/// job is on its way, and its answer is the only way to know the id: the task is left to end that request, and the
/// job is cancelled as soon as its id comes. Dropping is synchronous, so the cancel is a task of its own, kept in
/// `cancels` for the end of the program to wait for.
pub(super) struct OpenBatch<'a> {
    pub(super) heard: UnboundedReceiver<BatchEvent>,
    pub(super) job: AbortHandle,
    pub(super) model: Arc<dyn Llm>,
    pub(super) cancels: &'a Cancels,
}

impl Drop for OpenBatch<'_> {
    fn drop(&mut self) {
        let mut heard = std::mem::replace(&mut self.heard, tokio::sync::mpsc::unbounded_channel().1);
        let (mut open, mut sending): (Vec<String>, bool) = (Vec::new(), false);
        while let Ok(event) = heard.try_recv() {
            match event {
                BatchEvent::Creating { .. } => sending = true,
                BatchEvent::Submitted { batch_id, .. } => {
                    sending = false;
                    open.push(batch_id);
                }
                BatchEvent::Finished { batch_id, .. } => open.retain(|id| *id != batch_id),
                BatchEvent::Status { .. } => {}
            }
        }
        let creating = sending && !self.job.is_finished();
        if !creating {
            // nobody waits for the task any more: its poll stops here
            self.job.abort();
        }
        // without a runtime (the process is ending) there is nothing left to send the cancel with
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            self.job.abort();
            return;
        };
        let mut cancels = self.cancels.lock().unwrap_or_else(PoisonError::into_inner);
        if creating {
            let (model, job) = (self.model.clone(), self.job.clone());
            cancels.push(runtime.spawn(cancel_when_created(heard, job, model)));
        }
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

/// Waits for the id of the job whose creation was going on, stops the task as soon as it comes, and cancels the job;
/// all within `CANCEL_SECONDS`. A task that ends without a job (its creation was refused) leaves nothing to cancel.
async fn cancel_when_created(mut heard: UnboundedReceiver<BatchEvent>, job: AbortHandle, model: Arc<dyn Llm>) {
    let mut id: Option<String> = None;
    let within = tokio::time::timeout(Duration::from_secs(CANCEL_SECONDS), async {
        while let Some(event) = heard.recv().await {
            if let BatchEvent::Submitted { batch_id, .. } = event {
                job.abort();
                id = Some(batch_id.clone());
                return model.cancel_batch(&batch_id).await;
            }
        }
        Ok(())
    })
    .await;
    job.abort();
    match (within, id) {
        (Ok(Ok(())), _) => {}
        (Ok(Err(e)), Some(id)) => eprintln!("a-graph: cannot cancel the batch job {id} at the provider: {e}"),
        (Ok(Err(_)), None) => {}
        (Err(_), Some(id)) => eprintln!(
            "a-graph: cannot cancel the batch job {id} at the provider: no answer within {CANCEL_SECONDS} seconds"
        ),
        (Err(_), None) => eprintln!(
            "a-graph: a batch job was being created when the run stopped, and its id did not come within \
             {CANCEL_SECONDS} seconds: it may be left running at the provider"
        ),
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
