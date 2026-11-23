//! The HTTP side. `POST /run` answers when the graph is done. `POST /runs` starts it and answers at once with
//! the id of the run; `GET /runs/{id}/events` sends its events as they happen and `GET /runs/{id}` tells its state.

use std::collections::HashMap;
use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::time::Instant;

use axum::Router;
use axum::body::Bytes;
use axum::extract::{ConnectInfo, DefaultBodyLimit, FromRequest, Path as UrlPath, Request as HttpRequest, State};
use axum::http::StatusCode;
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{MethodFilter, MethodRouter, on};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, watch};

use super::api::Endpoint;
use super::rules::{SESSION_MARKER, valid_session};
use super::runs::{Next, Runs, SessionInUse, errors_json, run_within};
use super::{ServiceConfig, prepare};
use crate::error::{Error, Path, Phase, Rule};
use crate::logging::{TRACEPARENT, TraceContext};
use crate::run::{Program, build_with_env};

/// The most a request may weigh: a graph and, in base64, the files it reads.
const BODY_LIMIT: usize = 64 * 1024 * 1024;

/// How long the service waits, once told to stop, for the requests and the runs going on to end. Then the runs
/// still going on are stopped (`service_stopped`). Docker waits only 10 seconds by default (`stop_grace_period`)
/// before it kills.
const SHUTDOWN_SECONDS: u64 = 30;

/// How long the service waits after `SHUTDOWN_SECONDS` for the runs it stopped to close (their last event, their
/// trace, their log). Then it stops anyway and those still open are cut.
const CLOSE_SECONDS: u64 = 5;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    yml: String,
    message: String,
    session: Option<String>,
}

struct App {
    config: ServiceConfig,
    runs: Arc<Runs>,
    /// One permit for each run that may go on, held until the run ends.
    permits: Arc<Semaphore>,
    /// How many requests of each address have their body still arriving right now.
    reading: Arc<Reading>,
    /// The deadline of the shutdown, once the service is told to stop: the runs still going on are stopped then.
    deadline: watch::Receiver<Option<Instant>>,
}

/// How many requests of each address have their body still arriving; a place is held only for that, and given
/// back as soon as the body is read or refused, never for the rest of the request or the run.
#[derive(Default)]
struct Reading {
    per_address: Mutex<HashMap<IpAddr, usize>>,
}

impl Reading {
    /// A place for `addr`, if it has fewer than `cap` already; `None` otherwise.
    fn enter(self: &Arc<Self>, addr: IpAddr, cap: usize) -> Option<ReadingGuard> {
        let mut per_address = self.per_address.lock().unwrap();
        let n = per_address.entry(addr).or_insert(0);
        if *n >= cap {
            return None;
        }
        *n += 1;
        Some(ReadingGuard {
            reading: self.clone(),
            addr,
        })
    }
}

/// Gives the place of `addr` back when dropped, whatever ended its wait for the body.
struct ReadingGuard {
    reading: Arc<Reading>,
    addr: IpAddr,
}

impl Drop for ReadingGuard {
    fn drop(&mut self) {
        let mut per_address = self.reading.per_address.lock().unwrap();
        if let Some(n) = per_address.get_mut(&self.addr) {
            *n -= 1;
            if *n == 0 {
                per_address.remove(&self.addr);
            }
        }
    }
}

/// Listens on the port of the config until `shutdown` is ready. Then it takes no new request, and waits at most
/// `SHUTDOWN_SECONDS` for the requests and the runs going on (those of `POST /runs` too) to end. The runs still
/// going on are then stopped, and given at most `CLOSE_SECONDS` more to close.
pub async fn serve(config: ServiceConfig, shutdown: impl Future<Output = ()> + Send + 'static) -> Result<(), String> {
    let port = config.port;
    let max_runs = config.max_runs;
    let permits = Arc::new(Semaphore::new(max_runs));
    let runs = Runs::new(
        Duration::from_secs(config.retention_seconds),
        Duration::from_secs(config.run_timeout_seconds),
        config.data_dir.clone(),
        config.session_retention_seconds.map(Duration::from_secs),
    );
    let sweeping = runs.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            sweeping.sweep().await;
        }
    });
    // the routes are those the API describes, with their paths and methods
    let mut app = Router::new();
    for e in Endpoint::ALL {
        app = app.route(e.path(), handler(e));
    }
    // the time to stop is known when the signal comes
    let (told, mut deadline) = watch::channel(None::<Instant>);
    let app = app.layer(DefaultBodyLimit::max(BODY_LIMIT)).with_state(Arc::new(App {
        permits: permits.clone(),
        reading: Arc::new(Reading::default()),
        config,
        runs,
        deadline: deadline.clone(),
    }));
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .map_err(|e| format!("cannot listen on port {port}: {e}"))?;

    let signal = async move {
        shutdown.await;
        eprintln!("a-graph: shutting down");
        let _ = told.send(Some(Instant::now() + Duration::from_secs(SHUTDOWN_SECONDS)));
    };
    let server = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(signal)
        .into_future();
    // the server ends when every connection has ended, so a caller still connected when its run is stopped at the
    // deadline gets the answer of that run. A connection that does not end (a run that does not close) must not hold
    // the service past the time the runs have to close
    let late = async {
        match deadline.wait_for(Option::is_some).await {
            Ok(at) => {
                let at = at.unwrap_or_else(Instant::now) + Duration::from_secs(CLOSE_SECONDS);
                tokio::time::sleep_until(at).await;
            }
            Err(_) => std::future::pending().await,
        }
    };
    tokio::select! {
        served = server => served.map_err(|e| format!("the service stopped: {e}"))?,
        _ = late => {}
    }
    // every run going on holds a permit: they are all back when no run is left. The runs still going on at the
    // deadline are stopped then, and close themselves: they have a little more time for that
    let at = deadline.borrow().unwrap_or_else(Instant::now) + Duration::from_secs(CLOSE_SECONDS);
    while permits.available_permits() < max_runs && Instant::now() < at {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let cut = max_runs - permits.available_permits();
    if cut > 0 {
        eprintln!("a-graph: {cut} runs cut");
    }
    Ok(())
}

/// The handler of a route, on the method the API gives it.
fn handler(e: Endpoint) -> MethodRouter<Arc<App>> {
    let method = if e.method() == "post" {
        MethodFilter::POST
    } else {
        MethodFilter::GET
    };
    match e {
        Endpoint::Health => on(method, || async { Json200(json!({ "status": "ok" })) }),
        Endpoint::Run => on(method, run),
        Endpoint::Start => on(method, start),
        Endpoint::State => on(method, state),
        Endpoint::Events => on(method, events),
    }
}

struct Json200(Value);

impl IntoResponse for Json200 {
    fn into_response(self) -> Response {
        (StatusCode::OK, axum::Json(self.0)).into_response()
    }
}

/// `{ "errors": [ { "phase", "path", "rule", "message" } ] }` with this status.
fn failure(status: StatusCode, errors: &[Error]) -> Response {
    (status, axum::Json(errors_json(errors))).into_response()
}

fn bad_request(message: String) -> Response {
    failure(
        StatusCode::BAD_REQUEST,
        &[Error::new(Phase::Validation, Path::root(), Rule::BadValue, message)],
    )
}

/// Whether the files under this folder weigh more than `cap` bytes; a folder that is not there weighs 0. The walk
/// stops as soon as the sum passes the cap, and it does not follow symlinks.
fn folder_over(folder: &std::path::Path, cap: u64) -> std::io::Result<bool> {
    let mut total = 0u64;
    let mut pending = vec![folder.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            // a folder removed while we count weighs nothing
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e),
        };
        for entry in entries {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                pending.push(entry.path());
            } else if kind.is_file() {
                // a file removed while we count weighs nothing
                total += entry.metadata().map(|m| m.len()).unwrap_or(0);
                if total > cap {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

/// The program of a request, built, its message, the session it keeps in use and the permit; or the answer that
/// refuses the request. The permit is held by the blocking work, so a caller that hangs up during the setup does
/// not give it back before the setup ends.
async fn accept(
    app: &Arc<App>,
    body: Bytes,
    permit: OwnedSemaphorePermit,
) -> Result<(Program, String, SessionInUse, OwnedSemaphorePermit), Response> {
    // parsing, the disk and the build are blocking work: they do not run on a thread of the executor
    let work = tokio::task::spawn_blocking({
        let app = app.clone();
        move || setup(&app, body).map(|(program, message, using)| (program, message, using, permit))
    })
    .await;
    work.unwrap_or_else(|e| {
        Err(failure(
            StatusCode::INTERNAL_SERVER_ERROR,
            &[Error::new(
                Phase::Execution,
                Path::root(),
                Rule::Io,
                format!("cannot prepare the request: {e}"),
            )],
        ))
    })
}

/// The blocking part of `accept`. Each copy of the graph is dropped as soon as the next one is made: the body, the
/// text of `yml`, then the tree that is built.
fn setup(app: &App, body: Bytes) -> Result<(Program, String, SessionInUse), Response> {
    let config = &app.config;
    let request: Request = serde_json::from_slice(&body).map_err(|e| {
        bad_request(format!(
            "the request must be a JSON object with `yml`, `message` and, if wanted, `session`: {e}"
        ))
    })?;
    drop(body);
    let Request { yml, message, session } = request;
    let session = session.unwrap_or_else(|| "default".to_string());
    if !valid_session(&session) {
        return Err(bad_request(
            "`session` is a name of letters, digits, `-`, `_` and `.`, at most 64 characters".to_string(),
        ));
    }
    let root = prepare(&yml, config, &session).map_err(|errors| failure(StatusCode::UNPROCESSABLE_ENTITY, &errors))?;
    drop(yml);
    // in use before the folder exists, so the sweep cannot delete it from under the request
    let mut using = app.runs.enter(&session);
    let folder = config.data_dir.join(&session);
    // the folder is there before the build, which may open files in it (the log)
    std::fs::create_dir_all(&folder).map_err(|e| {
        failure(
            StatusCode::INTERNAL_SERVER_ERROR,
            &[Error::new(
                Phase::Execution,
                Path::root(),
                Rule::Io,
                format!("cannot create the folder of the session: {e}"),
            )],
        )
    })?;
    // the sweep deletes only the folders with this mark, so it never touches what the service did not make; it is
    // only made here, not stamped: a refused request does not keep the session alive
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(folder.join(SESSION_MARKER))
        .map_err(|e| {
            failure(
                StatusCode::INTERNAL_SERVER_ERROR,
                &[Error::new(
                    Phase::Execution,
                    Path::root(),
                    Rule::Io,
                    format!("cannot mark the folder of the session: {e}"),
                )],
            )
        })?;
    // the program is built again for each request: requests share nothing
    let program = build_with_env(root, &config.allowed_env)
        .map_err(|errors| failure(StatusCode::UNPROCESSABLE_ENTITY, &errors.0))?;
    if let Some(cap) = config.session_max_bytes {
        let over = folder_over(&folder, cap).map_err(|e| {
            failure(
                StatusCode::INTERNAL_SERVER_ERROR,
                &[Error::new(
                    Phase::Execution,
                    Path::root(),
                    Rule::Io,
                    format!("cannot weigh the folder of the session: {e}"),
                )],
            )
        })?;
        if over {
            let message = format!(
                "the session `{session}` weighs more than session_max_bytes ({cap}): use another session, or ask the operator of the service to free it"
            );
            return Err(failure(
                StatusCode::UNPROCESSABLE_ENTITY,
                &[Error::new(Phase::Execution, Path::root(), Rule::SessionFull, message)],
            ));
        }
    }
    using.accepted();
    Ok((program, message, using))
}

/// The answer that refuses a request when every permit to run is taken (503).
fn busy(max_runs: usize) -> Response {
    let message = format!("the service is running its most runs (max_runs: {max_runs}): try again later");
    failure(
        StatusCode::SERVICE_UNAVAILABLE,
        &[Error::new(Phase::Execution, Path::root(), Rule::TooManyRuns, message)],
    )
}

/// The answer that refuses a request when its address already has `max_reading_per_address` whose body is
/// still arriving (503), even if `max_runs` has a free place.
fn busy_address(cap: usize) -> Response {
    let message = format!(
        "this address already has its most requests whose body is arriving (max_reading_per_address: {cap}): try again later"
    );
    failure(
        StatusCode::SERVICE_UNAVAILABLE,
        &[Error::new(Phase::Execution, Path::root(), Rule::TooManyRuns, message)],
    )
}

/// The body of a request, read within `read_timeout_seconds`. It is read as `Bytes`, so the limit of
/// `DefaultBodyLimit` and the answers of axum to a body too large (413) or cut (400) are those of the extractor; a
/// body still arriving at the deadline is refused (408), and the connection is closed with the answer.
async fn read_body(app: &App, request: HttpRequest) -> Result<Bytes, Response> {
    let seconds = app.config.read_timeout_seconds;
    match tokio::time::timeout(Duration::from_secs(seconds), Bytes::from_request(request, &())).await {
        Ok(Ok(body)) => Ok(body),
        Ok(Err(rejection)) => Err(rejection.into_response()),
        Err(_) => Err(failure(
            StatusCode::REQUEST_TIMEOUT,
            &[Error::new(
                Phase::Execution,
                Path::root(),
                Rule::ReadTimeout,
                format!("the request did not arrive within read_timeout_seconds ({seconds})"),
            )],
        )),
    }
}

async fn run(
    State(app): State<Arc<App>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: HttpRequest,
) -> Response {
    // taken before the permit of `max_runs`: an address already at its cap costs the service nothing
    let Some(reading) = app.reading.enter(addr.ip(), app.config.max_reading_per_address) else {
        return busy_address(app.config.max_reading_per_address);
    };
    // taken without waiting and before the body is read, so a request that is refused costs nothing; held while the
    // body arrives and until the run ends, or dropped with a request that is not accepted
    let Ok(permit) = app.permits.clone().try_acquire_owned() else {
        return busy(app.config.max_runs);
    };
    // read before the body, which takes the request
    let caller = TraceContext::from_headers(request.headers());
    let body = match read_body(&app, request).await {
        Ok(body) => body,
        Err(refusal) => return refusal,
    };
    // the address has its place back: the rest of the request and the run do not count against its cap
    drop(reading);
    let (program, message, using, permit) = match accept(&app, body, permit).await {
        Ok(x) => x,
        Err(refusal) => return refusal,
    };
    let traceparent = program.join_trace(caller);
    // the run is a task of its own that holds the permit and the session: a caller that hangs up drops this handler,
    // not the run, so the permit is given back, and the session can be swept, only when the run really ends (or is
    // stopped by its timeout or the shutdown)
    let timeout = Duration::from_secs(app.config.run_timeout_seconds);
    let deadline = app.deadline.clone();
    let task = tokio::spawn(async move {
        let _permit = permit;
        let _using = using;
        run_within(program, &message, timeout, deadline).await
    });
    let outcome = task.await.unwrap_or_else(|e| {
        Err(Error::new(
            Phase::Execution,
            Path::root(),
            Rule::Io,
            format!("the run stopped: {e}"),
        ))
    });
    let answer = match outcome {
        Ok(output) => Json200(output.to_json()).into_response(),
        Err(e) => failure(StatusCode::BAD_GATEWAY, &[e]),
    };
    with_traceparent(answer, &traceparent)
}

/// The answer with the `traceparent` of its run, so that the caller finds the run in its trace.
fn with_traceparent(mut answer: Response, traceparent: &str) -> Response {
    // a traceparent is ASCII hex and dashes, always a valid header value
    if let Ok(value) = axum::http::HeaderValue::from_str(traceparent) {
        answer.headers_mut().insert(TRACEPARENT, value);
    }
    answer
}

async fn start(
    State(app): State<Arc<App>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: HttpRequest,
) -> Response {
    let Some(reading) = app.reading.enter(addr.ip(), app.config.max_reading_per_address) else {
        return busy_address(app.config.max_reading_per_address);
    };
    let Ok(permit) = app.permits.clone().try_acquire_owned() else {
        return busy(app.config.max_runs);
    };
    let caller = TraceContext::from_headers(request.headers());
    let body = match read_body(&app, request).await {
        Ok(body) => body,
        Err(refusal) => return refusal,
    };
    drop(reading);
    let (program, message, using, permit) = match accept(&app, body, permit).await {
        Ok(x) => x,
        Err(refusal) => return refusal,
    };
    let traceparent = program.join_trace(caller);
    let id = app.runs.start(program, message, permit, app.deadline.clone(), using);
    with_traceparent(
        (StatusCode::ACCEPTED, axum::Json(json!({ "run": id }))).into_response(),
        &traceparent,
    )
}

fn not_found(id: &str) -> Response {
    failure(
        StatusCode::NOT_FOUND,
        &[Error::new(
            Phase::Execution,
            Path::root(),
            Rule::UnknownRef,
            format!("there is no run `{id}` (it never existed, or it ended long ago)"),
        )],
    )
}

async fn state(State(app): State<Arc<App>>, UrlPath(id): UrlPath<String>) -> Response {
    match app.runs.get(&id) {
        Some(run) => Json200(run.status()).into_response(),
        None => not_found(&id),
    }
}

/// The events already happened, then the new ones as they come, and at the end `result` or `error`.
async fn events(State(app): State<Arc<App>>, UrlPath(id): UrlPath<String>) -> Response {
    let Some(run) = app.runs.get(&id) else {
        return not_found(&id);
    };
    let stream = futures_util::stream::unfold((run, 0usize, false), |(run, index, finished)| async move {
        if finished {
            return None;
        }
        loop {
            // ask to be told of a change BEFORE looking, so a change in between is not missed; the wait
            // ends before anything is returned, because the wait borrows the run
            let step = {
                let changed = run.changed();
                tokio::pin!(changed);
                changed.as_mut().enable();
                match run.next(index) {
                    Next::Wait => {
                        changed.await;
                        None
                    }
                    found => Some(found),
                }
            };
            match step {
                Some(Next::Event(name, data)) => {
                    let event: Result<SseEvent, Infallible> =
                        Ok(SseEvent::default().event(name).data(data.to_string()));
                    return Some((event, (run, index + 1, false)));
                }
                Some(Next::Dropped(count)) => {
                    // the watcher came after these were dropped: it is told how many, and goes on from the first kept
                    let event: Result<SseEvent, Infallible> = Ok(SseEvent::default()
                        .event("dropped")
                        .data(json!({ "count": count }).to_string()));
                    return Some((event, (run, index + count, false)));
                }
                Some(Next::Last(name, data)) => {
                    let event: Result<SseEvent, Infallible> =
                        Ok(SseEvent::default().event(name).data(data.to_string()));
                    return Some((event, (run, index, true)));
                }
                _ => {}
            }
        }
    });
    Sse::new(stream).keep_alive(KeepAlive::default()).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each address has its own places, and a place comes back when its guard is dropped.
    #[test]
    fn reading_per_address() {
        let reading = Arc::new(Reading::default());
        let (a, b): (IpAddr, IpAddr) = ("10.0.0.1".parse().unwrap(), "10.0.0.2".parse().unwrap());
        let first = reading.enter(a, 2).unwrap();
        let _second = reading.enter(a, 2).unwrap();
        assert!(reading.enter(a, 2).is_none());
        // another address is not affected
        let other = reading.enter(b, 2);
        assert!(other.is_some());
        drop(first);
        assert!(reading.enter(a, 2).is_some());
        // an address without requests is forgotten
        drop(other);
        assert!(!reading.per_address.lock().unwrap().contains_key(&b));
    }
}
