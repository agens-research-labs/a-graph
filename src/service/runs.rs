//! The runs that are going on or just finished: what a caller can watch and ask for after it started one.
//! A run keeps its events and its outcome in memory for a while after the end, then it is forgotten.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tokio::sync::{Notify, OwnedSemaphorePermit};

use crate::error::Error;
use crate::logging::stream_event;
use crate::run::Program;

pub(super) enum Outcome {
    Done(Value),
    Failed(Value),
}

/// What the stream has to send next.
pub(super) enum Next {
    Event(String, Value),
    /// The last one: `result` or `error`.
    Last(String, Value),
    Wait,
}

pub(super) struct RunState {
    inner: Mutex<Inner>,
    notify: Notify,
}

struct Inner {
    events: Vec<(String, Value)>,
    outcome: Option<Outcome>,
    finished: Option<Instant>,
}

impl RunState {
    fn new() -> Arc<Self> {
        Arc::new(RunState { inner: Mutex::new(Inner { events: Vec::new(), outcome: None, finished: None }), notify: Notify::new() })
    }

    fn push(&self, name: String, data: Value) {
        self.inner.lock().unwrap().events.push((name, data));
        self.notify.notify_waiters();
    }

    fn finish(&self, outcome: Outcome) {
        let mut inner = self.inner.lock().unwrap();
        inner.outcome = Some(outcome);
        inner.finished = Some(Instant::now());
        drop(inner);
        self.notify.notify_waiters();
    }

    /// The event number `index`, or the last one, or nothing yet.
    pub(super) fn next(&self, index: usize) -> Next {
        let inner = self.inner.lock().unwrap();
        if let Some((name, data)) = inner.events.get(index) {
            return Next::Event(name.clone(), data.clone());
        }
        match &inner.outcome {
            Some(Outcome::Done(v)) => Next::Last("result".into(), v.clone()),
            Some(Outcome::Failed(v)) => Next::Last("error".into(), v.clone()),
            None => Next::Wait,
        }
    }

    /// A future that is ready when something changes; to be made BEFORE `next`, so nothing is missed.
    pub(super) fn changed(&self) -> tokio::sync::futures::Notified<'_> {
        self.notify.notified()
    }

    /// `{ "state": "running" | "done" | "failed", "result" | "errors" }`.
    pub(super) fn status(&self) -> Value {
        match &self.inner.lock().unwrap().outcome {
            None => json!({ "state": "running" }),
            Some(Outcome::Done(v)) => json!({ "state": "done", "result": v }),
            Some(Outcome::Failed(v)) => json!({ "state": "failed", "errors": v["errors"] }),
        }
    }
}

pub(super) struct Runs {
    map: Mutex<HashMap<String, Arc<RunState>>>,
    retention: Duration,
}

impl Runs {
    pub(super) fn new(retention: Duration) -> Arc<Self> {
        Arc::new(Runs { map: Mutex::new(HashMap::new()), retention })
    }

    pub(super) fn get(&self, id: &str) -> Option<Arc<RunState>> {
        self.map.lock().unwrap().get(id).cloned()
    }

    /// Forgets the runs that ended more than `retention` ago.
    pub(super) fn sweep(&self) {
        let retention = self.retention;
        self.map.lock().unwrap().retain(|_, s| s.inner.lock().unwrap().finished.is_none_or(|t| t.elapsed() < retention));
    }

    /// Starts the run and gives back its id at once. The events the graph asks to show (its `verbose`)
    /// are kept as the run goes; the outcome is kept when it ends. The permit is given back at the end.
    pub(super) fn start(self: &Arc<Self>, mut program: Program, message: String, permit: OwnedSemaphorePermit) -> String {
        let id = format!("r-{}", &uuid::Uuid::new_v4().simple().to_string()[..16]);
        let state = RunState::new();
        self.map.lock().unwrap().insert(id.clone(), state.clone());

        let (level, allow) = (program.verbose.level(), program.verbose.allow_content());
        let hear = state.clone();
        program.observer = Some(Arc::new(move |ev| {
            if let Some((name, data)) = stream_event(ev, level, allow) {
                hear.push(name, data);
            }
        }));
        tokio::spawn(async move {
            let _permit = permit;
            let program = Arc::new(program);
            let outcome = program.run(&message).await;
            if let Some(p) = Arc::into_inner(program) {
                p.logger.close().await;
            }
            state.finish(match outcome {
                Ok(output) => Outcome::Done(output.to_json()),
                Err(e) => Outcome::Failed(errors_json(&[e])),
            });
        });
        id
    }
}

/// `{ "errors": [ { "phase", "path", "rule", "message" } ] }`.
pub(super) fn errors_json(errors: &[Error]) -> Value {
    let list: Vec<Value> = errors.iter().map(|e| json!({ "phase": e.phase.to_string(), "path": e.path.to_string(), "rule": e.rule.code(), "message": e.message })).collect();
    json!({ "errors": list })
}
