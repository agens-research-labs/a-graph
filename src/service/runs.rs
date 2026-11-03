//! The runs that are going on or just finished: what a caller can watch and ask for after it started one.
//! A run keeps its events and its outcome in memory for a while after the end, then it is forgotten.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tokio::sync::{Notify, OwnedSemaphorePermit, watch};

use super::rules::{SESSION_MARKER, valid_session};
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
        Arc::new(RunState {
            inner: Mutex::new(Inner {
                events: VecDeque::new(),
                dropped: 0,
                max_events,
                outcome: None,
                finished: None,
            }),
            notify: Notify::new(),
        })
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

/// The end of the name of a session folder that is being deleted; `~` is not allowed in a session name.
const DELETING: &str = "~deleting";

/// A name in `data_dir` that ends with `DELETING` and is not taken, for the folder of `session`.
fn aside(data_dir: &std::path::Path, session: &str) -> PathBuf {
    let mut n = 0u32;
    loop {
        let name = if n == 0 {
            format!("{session}{DELETING}")
        } else {
            format!("{session}~{n}{DELETING}")
        };
        let path = data_dir.join(name);
        if std::fs::symlink_metadata(&path).is_err() {
            return path;
        }
        n += 1;
    }
}

/// Whether the folder holds the mark of the service, a real file (not a link).
fn marked(folder: &std::path::Path) -> bool {
    std::fs::symlink_metadata(folder.join(SESSION_MARKER)).is_ok_and(|m| m.is_file())
}

/// Deletes a folder set aside; one already gone is not an error.
fn remove(folder: &std::path::Path, session: &str) {
    match remove_mark_last(folder) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => eprintln!("a-graph: cannot remove session {session}: {e}"),
    }
}

/// Deletes what the folder holds, then its mark, then the folder. A deletion cut short leaves the mark, so the next
/// scan still knows the folder as one of the service and finishes it.
fn remove_mark_last(folder: &std::path::Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(folder)? {
        let entry = entry?;
        if entry.file_name() == SESSION_MARKER {
            continue;
        }
        // a link is removed, not followed
        if entry.file_type()?.is_dir() {
            std::fs::remove_dir_all(entry.path())?;
        } else {
            std::fs::remove_file(entry.path())?;
        }
    }
    match std::fs::remove_file(folder.join(SESSION_MARKER)) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e),
        _ => {}
    }
    std::fs::remove_dir(folder)
}

/// The folders of the sessions are looked at no more often than this, unless the retention is shorter.
const SCAN_EVERY: Duration = Duration::from_secs(10);

pub(super) struct Runs {
    map: Mutex<HashMap<String, Arc<RunState>>>,
    retention: Duration,
    /// The longest a run may last.
    run_timeout: Duration,
    data_dir: PathBuf,
    /// How long a session folder is kept after its last request; `None`: for ever.
    session_retention: Option<Duration>,
    /// The sessions that have runs going on, with how many; by their name in lowercase (`in_use_key`).
    sessions: Mutex<HashMap<String, usize>>,
    last_scan: Mutex<Instant>,
}

/// The key of a session among those in use: its name in lowercase. On a filesystem that ignores case, `S` and `s`
/// are one folder, so a run on either keeps that folder from the sweep; elsewhere they are two folders, and a run on
/// one only delays the sweep of the other.
fn in_use_key(session: &str) -> String {
    session.to_ascii_lowercase()
}

/// A session with a run going on: its folder is not deleted while this lives.
pub(super) struct SessionInUse {
    runs: Arc<Runs>,
    /// The name of the session, which is the name of its folder.
    session: String,
    /// Its key among the sessions in use.
    key: String,
    /// Whether the request was accepted: only then does it count as a use of the session.
    accepted: bool,
}

impl SessionInUse {
    /// The request is accepted: the session is stamped now, and again when this is dropped.
    pub(super) fn accepted(&mut self) {
        self.accepted = true;
        stamp(&self.runs.data_dir.join(&self.session), &self.session);
    }
}

impl Drop for SessionInUse {
    fn drop(&mut self) {
        let mut sessions = self.runs.sessions.lock().unwrap();
        if let Some(n) = sessions.get_mut(&self.key) {
            *n -= 1;
            if *n == 0 {
                sessions.remove(&self.key);
            }
        }
        // the run was part of the life of the folder: its time is counted from the end; a refused request is not
        if self.accepted {
            stamp(&self.runs.data_dir.join(&self.session), &self.session);
        }
    }
}

/// Sets the modified time of the mark of the session folder to now: the sweep counts the idle time from it. The mark
/// is emptied, not given a time, because that needs only the right to write it, not to own it. A failure is written
/// on the standard error: the session may then be deleted too early.
fn stamp(folder: &std::path::Path, session: &str) {
    if let Err(e) = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(folder.join(SESSION_MARKER))
    {
        eprintln!("a-graph: cannot stamp session {session}: {e}");
    }
}

impl Runs {
    pub(super) fn new(
        retention: Duration,
        run_timeout: Duration,
        data_dir: PathBuf,
        session_retention: Option<Duration>,
    ) -> Arc<Self> {
        Arc::new(Runs {
            map: Mutex::new(HashMap::new()),
            retention,
            run_timeout,
            data_dir,
            session_retention,
            sessions: Mutex::new(HashMap::new()),
            last_scan: Mutex::new(Instant::now()),
        })
    }

    /// Marks the session as in use until the result is dropped.
    pub(super) fn enter(self: &Arc<Self>, session: &str) -> SessionInUse {
        let key = in_use_key(session);
        *self.sessions.lock().unwrap().entry(key.clone()).or_insert(0) += 1;
        SessionInUse {
            runs: self.clone(),
            session: session.to_string(),
            key,
            accepted: false,
        }
    }

    pub(super) fn get(&self, id: &str) -> Option<Arc<RunState>> {
        self.map.lock().unwrap().get(id).cloned()
    }

    /// Forgets the runs that ended more than `retention` ago, and deletes the sessions nobody uses.
    pub(super) async fn sweep(self: &Arc<Self>) {
        let retention = self.retention;
        self.map
            .lock()
            .unwrap()
            .retain(|_, s| s.inner.lock().unwrap().finished.is_none_or(|t| t.elapsed() < retention));
        if let Some(keep) = self.session_retention {
            let due = {
                let mut last = self.last_scan.lock().unwrap();
                let due = last.elapsed() >= SCAN_EVERY.min(keep);
                if due {
                    *last = Instant::now();
                }
                due
            };
            if due {
                // the scan and the deletions read and write the disk: they do not run on a thread of the executor
                let runs = self.clone();
                if let Err(e) = tokio::task::spawn_blocking(move || runs.delete_old_sessions(keep)).await {
                    eprintln!("a-graph: the sweep of the sessions stopped: {e}");
                }
            }
        }
    }

    /// Deletes the folders of `data_dir` whose mark was not stamped for `keep` and with no run going on, and finishes the
    /// deletions a stop cut short.
    fn delete_old_sessions(&self, keep: Duration) {
        let entries = match std::fs::read_dir(&self.data_dir) {
            Ok(entries) => entries,
            // no request has made the folder yet
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return,
            Err(e) => return eprintln!("a-graph: cannot read the sessions in {}: {e}", self.data_dir.display()),
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("a-graph: cannot read the sessions in {}: {e}", self.data_dir.display());
                    continue;
                }
            };
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.file_type().is_ok_and(|t| t.is_dir());
            // a folder already set aside to be deleted, by this scan or by one that was stopped halfway
            if name.ends_with(DELETING) {
                if is_dir {
                    if marked(&entry.path()) {
                        remove(&entry.path(), &name);
                    } else {
                        // without the mark it is either a folder the service did not make, which stays, or one whose
                        // deletion stopped right after the mark, which is empty: only an empty folder is removed
                        let _ = std::fs::remove_dir(entry.path());
                    }
                }
                continue;
            }
            // only a real folder (not a link) with the name of a session and the mark of the service
            if !valid_session(&name) || !is_dir || !marked(&entry.path()) {
                continue;
            }
            // under the lock no run can start or end on the folder between the look and the rename; the rename is
            // quick, the deletion is not, so it happens after the lock. A renamed folder is no longer a session:
            // a request for its name makes a new one, and a deletion cut short leaves no half folder behind it
            let doomed = {
                let sessions = self.sessions.lock().unwrap();
                let old = std::fs::symlink_metadata(entry.path().join(SESSION_MARKER))
                    .and_then(|m| m.modified())
                    .is_ok_and(|t| t.elapsed().is_ok_and(|age| age > keep));
                if !old || sessions.contains_key(&in_use_key(&name)) {
                    continue;
                }
                let doomed = aside(&self.data_dir, &name);
                if let Err(e) = std::fs::rename(entry.path(), &doomed) {
                    eprintln!("a-graph: cannot remove session {name}: {e}");
                    continue;
                }
                doomed
            };
            remove(&doomed, &name);
        }
    }

    /// Starts the run and gives back its id at once. The events the graph asks to show (its `verbose`)
    /// are kept as the run goes; the outcome is kept when it ends. The permit is given back, and the session
    /// let go, at the end. The run is stopped at the deadline of the shutdown, once `deadline` tells it.
    pub(super) fn start(
        self: &Arc<Self>,
        mut program: Program,
        message: String,
        permit: OwnedSemaphorePermit,
        deadline: watch::Receiver<Option<tokio::time::Instant>>,
        using: SessionInUse,
    ) -> String {
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
            // the run is a task of its own, so that a panic in it still ends the run, as failed
            let outcome = tokio::spawn(async move { run_within(program, &message, run_timeout, deadline).await })
                .await
                .unwrap_or_else(|e| {
                    Err(Error::new(
                        Phase::Execution,
                        Path::root(),
                        Rule::Io,
                        format!("the run stopped: {e}"),
                    ))
                });
            state.finish(match outcome {
                Ok(output) => Outcome::Done(output.to_json()),
                Err(e) => Outcome::Failed(errors_json(&[e])),
            });
            drop(using);
        });
        id
    }
}

/// Runs the program to its end, or until `timeout` has passed (`run_timeout`), or until the deadline of the shutdown
/// of the service, once there is one (`service_stopped`); then closes its log.
pub(super) async fn run_within(
    program: Program,
    message: &str,
    timeout: Duration,
    mut deadline: watch::Receiver<Option<tokio::time::Instant>>,
) -> Result<RunOutput, Error> {
    let program = Arc::new(program);
    // when the time is up the nodes are stopped, and the run still ends as always (its last event, its trace)
    let late = async move {
        // the deadline is sent before its sender is dropped, so it is still read after
        match deadline.wait_for(Option::is_some).await.map(|at| *at) {
            Ok(Some(at)) => tokio::time::sleep_until(at).await,
            _ => std::future::pending().await,
        }
    };
    let stop = async move {
        tokio::select! {
            _ = tokio::time::sleep(timeout) => Error::new(Phase::Execution, Path::root(), Rule::RunTimeout, format!("the run did not end within {} seconds (run_timeout_seconds)", timeout.as_secs())),
            _ = late => Error::new(Phase::Execution, Path::root(), Rule::ServiceStopped, "the service was stopped before the run ended"),
        }
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
    use crate::models::{
        BatchEvents, BatchRequest, BatchReturn, Llm, ModelError, ModelInvocation, ModelRequirements, ModelResponse,
    };

    /// A model that panics when it is called.
    struct Panics;

    #[async_trait::async_trait]
    impl Llm for Panics {
        fn check(&self, _: &ModelRequirements) -> Result<(), ModelError> {
            Ok(())
        }

        async fn invoke(&self, _: &ModelInvocation) -> Result<ModelResponse, ModelError> {
            panic!("the model panicked");
        }

        async fn invoke_batch(&self, _: Vec<BatchRequest>, _: BatchEvents) -> BatchReturn {
            unreachable!("no batch here")
        }
    }

    /// A run started with `POST /runs` whose model panics ends as failed, with the error of `POST /run`, instead of
    /// staying `running` for ever.
    #[tokio::test]
    async fn run_panicked_failed() {
        let mut program = crate::run::build(include_str!("../../tests/service/run_panic/graph.yml"))
            .unwrap_or_else(|e| panic!("{e}"));
        program.models.insert("m", Box::new(Panics));
        let runs = Runs::new(
            Duration::from_secs(60),
            Duration::from_secs(60),
            PathBuf::from("target/test-service/run_panic"),
            None,
        );
        let permit = Arc::new(tokio::sync::Semaphore::new(1)).try_acquire_owned().unwrap();
        let (_stop, deadline) = watch::channel(None);
        let using = runs.enter("s");
        let id = runs.start(program, "x".into(), permit, deadline, using);
        let state = runs.get(&id).unwrap();
        let waited = Instant::now();
        while state.status()["state"] == "running" {
            assert!(waited.elapsed() < Duration::from_secs(5), "the run is still running");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let status = state.status();
        assert_eq!(status["state"], "failed", "{status}");
        assert_eq!(status["errors"][0]["rule"], "io", "{status}");
        let message = status["errors"][0]["message"].as_str().unwrap();
        assert!(message.starts_with("the run stopped:"), "{status}");
    }

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
        assert_eq!(
            names(&state, 0),
            ["dropped 2", "e2", "e3", "e4"],
            "a watcher from the start is told what it missed"
        );
        assert_eq!(names(&state, 1), ["dropped 1", "e2", "e3", "e4"]);
        assert_eq!(
            names(&state, 3),
            ["e3", "e4"],
            "a watcher after the dropped ones misses nothing"
        );
        state.finish(Outcome::Done(json!({ "items": [] })));
        assert_eq!(names(&state, 0), ["dropped 2", "e2", "e3", "e4", "result"]);
    }
}
