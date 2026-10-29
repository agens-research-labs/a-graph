//! The runs that are going on or just finished: what a caller can watch and ask for after it started one.
//! A run keeps its events and its outcome in memory for a while after the end, then it is forgotten.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tokio::sync::{Notify, OwnedSemaphorePermit};

use crate::error::{Error, Path, Phase, Rule};
use crate::logging::stream_event;
use crate::run::{Program, RunOutput};

/// The most events a run keeps: beyond it, the oldest are dropped, so a long run does not fill the memory.
const MAX_EVENTS: usize = 1000;

pub(super) enum Outcome {
    Done(Value),
    Failed(Value),
}

/// What the stream has to send next.
pub(super) enum Next {
    Event(String, Value),
    /// The last one: `result` or `error`.
    Last(String, Value),
    /// The events from the index asked for were dropped, this many: the stream goes on after them.
    Dropped(usize),
    Wait,
}

pub(super) struct RunState {
    inner: Mutex<Inner>,
    notify: Notify,
}

struct Inner {
    /// The events kept; the first has the index `dropped`.
    events: VecDeque<(String, Value)>,
    /// How many events were dropped, the oldest, to keep at most `max_events`.
    dropped: usize,
    max_events: usize,
    outcome: Option<Outcome>,
    finished: Option<Instant>,
}

impl RunState {
    fn new(max_events: usize) -> Arc<Self> {
        Arc::new(RunState { inner: Mutex::new(Inner { events: VecDeque::new(), dropped: 0, max_events, outcome: None, finished: None }), notify: Notify::new() })
    }

    fn push(&self, name: String, data: Value) {
        let mut inner = self.inner.lock().unwrap();
        inner.events.push_back((name, data));
        if inner.events.len() > inner.max_events {
            inner.events.pop_front();
            inner.dropped += 1;
        }
        drop(inner);
        self.notify.notify_waiters();
    }

    fn finish(&self, outcome: Outcome) {
        let mut inner = self.inner.lock().unwrap();
        inner.outcome = Some(outcome);
        inner.finished = Some(Instant::now());
        drop(inner);
        self.notify.notify_waiters();
    }

    /// The event number `index` (counted from the first of the run, dropped or not), or how many were dropped
    /// from there, or the last one, or nothing yet.
    pub(super) fn next(&self, index: usize) -> Next {
        let inner = self.inner.lock().unwrap();
        if index < inner.dropped {
            return Next::Dropped(inner.dropped - index);
        }
        if let Some((name, data)) = inner.events.get(index - inner.dropped) {
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
    /// The longest a run may last.
    run_timeout: Duration,
}

impl Runs {
    pub(super) fn new(retention: Duration, run_timeout: Duration) -> Arc<Self> {
        Arc::new(Runs { map: Mutex::new(HashMap::new()), retention, run_timeout })
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
        let state = RunState::new(MAX_EVENTS);
        self.map.lock().unwrap().insert(id.clone(), state.clone());

        let (level, allow) = (program.verbose.level(), program.verbose.allow_content());
        let hear = state.clone();
        program.observer = Some(Arc::new(move |ev| {
            if let Some((name, data)) = stream_event(ev, level, allow) {
                hear.push(name, data);
            }
        }));
        let run_timeout = self.run_timeout;
        tokio::spawn(async move {
            let _permit = permit;
            let outcome = run_within(program, &message, run_timeout).await;
            state.finish(match outcome {
                Ok(output) => Outcome::Done(output.to_json()),
                Err(e) => Outcome::Failed(errors_json(&[e])),
            });
        });
        id
    }
}

/// Runs the program to its end, or until `timeout` has passed (`run_timeout`); then closes its log.
pub(super) async fn run_within(program: Program, message: &str, timeout: Duration) -> Result<RunOutput, Error> {
    let program = Arc::new(program);
    // when the time is up the nodes are stopped, and the run still ends as always (its last event, its trace)
    let stop = async move {
        tokio::time::sleep(timeout).await;
        Error::new(Phase::Execution, Path::root(), Rule::RunTimeout, format!("the run did not end within {} seconds (run_timeout_seconds)", timeout.as_secs()))
    };
    let outcome = program.run_until(message, stop).await;
    // the branches of a stopped run are aborted and let go of the program a moment later: wait for them, at most a second
    let mut program = program;
    for _ in 0..100 {
        match Arc::try_unwrap(program) {
            Ok(p) => {
                p.logger.close().await;
                break;
            }
            Err(still) => {
                program = still;
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }
    outcome
}

/// `{ "errors": [ { "phase", "path", "rule", "message" } ] }`.
pub(super) fn errors_json(errors: &[Error]) -> Value {
    let list: Vec<Value> = errors.iter().map(|e| json!({ "phase": e.phase.to_string(), "path": e.path.to_string(), "rule": e.rule.code(), "message": e.message })).collect();
    json!({ "errors": list })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(state: &RunState, mut index: usize) -> Vec<String> {
        let mut seen = Vec::new();
        loop {
            match state.next(index) {
                Next::Event(name, _) => {
                    seen.push(name);
                    index += 1;
                }
                Next::Dropped(count) => {
                    seen.push(format!("dropped {count}"));
                    index += count;
                }
                Next::Last(name, _) => {
                    seen.push(name);
                    return seen;
                }
                Next::Wait => return seen,
            }
        }
    }

    #[test]
    fn events_capped() {
        let state = RunState::new(3);
        for name in ["e0", "e1", "e2"] {
            state.push(name.into(), json!({}));
        }
        assert_eq!(names(&state, 0), ["e0", "e1", "e2"], "up to the cap nothing is dropped");
        state.push("e3".into(), json!({}));
        state.push("e4".into(), json!({}));
        assert_eq!(names(&state, 0), ["dropped 2", "e2", "e3", "e4"], "a watcher from the start is told what it missed");
        assert_eq!(names(&state, 1), ["dropped 1", "e2", "e3", "e4"]);
        assert_eq!(names(&state, 3), ["e3", "e4"], "a watcher after the dropped ones misses nothing");
        state.finish(Outcome::Done(json!({ "items": [] })));
        assert_eq!(names(&state, 0), ["dropped 2", "e2", "e3", "e4", "result"]);
    }
}
