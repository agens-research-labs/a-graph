//! The HTTP side. `POST /run` answers when the graph is done. `POST /runs` starts it and answers at once with
//! the id of the run; `GET /runs/{id}/events` sends its events as they happen and `GET /runs/{id}` tells its state.

use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::{DefaultBodyLimit, Path as UrlPath, State};
use axum::http::StatusCode;
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{MethodFilter, MethodRouter, on};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::Semaphore;

use super::api::Endpoint;
use super::runs::{Next, Runs, errors_json, run_within};
use super::{ServiceConfig, prepare};
use crate::error::{Error, Path, Phase, Rule};
use crate::run::{Program, build};

/// The most a request may weigh: a graph and, in base64, the files it reads.
const BODY_LIMIT: usize = 64 * 1024 * 1024;

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
}

/// Listens on the port of the config until the process ends.
pub async fn serve(config: ServiceConfig) -> Result<(), String> {
    let port = config.port;
    let runs = Runs::new(Duration::from_secs(config.retention_seconds), Duration::from_secs(config.run_timeout_seconds));
    let sweeping = runs.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            sweeping.sweep();
        }
    });
    // the routes are those the API describes, with their paths and methods
    let mut app = Router::new();
    for e in Endpoint::ALL {
        app = app.route(e.path(), handler(e));
    }
    let app = app
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .with_state(Arc::new(App { permits: Arc::new(Semaphore::new(config.max_runs)), config, runs }));
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.map_err(|e| format!("cannot listen on port {port}: {e}"))?;
    axum::serve(listener, app).await.map_err(|e| format!("the service stopped: {e}"))
}

/// The handler of a route, on the method the API gives it.
fn handler(e: Endpoint) -> MethodRouter<Arc<App>> {
    let method = if e.method() == "post" { MethodFilter::POST } else { MethodFilter::GET };
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
    failure(StatusCode::BAD_REQUEST, &[Error::new(Phase::Validation, Path::root(), Rule::BadValue, message)])
}

/// A session is a folder name: letters, digits, `-`, `_` and `.`, at most 64, and not made of dots only.
fn valid_session(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')) && s.chars().any(|c| c != '.')
}

/// The program of a request, built, and its message; or the answer that refuses the request.
fn accept(config: &ServiceConfig, body: &[u8]) -> Result<(Program, String), Response> {
    let request: Request = serde_json::from_slice(body)
        .map_err(|e| bad_request(format!("the request must be a JSON object with `yml`, `message` and, if wanted, `session`: {e}")))?;
    let session = request.session.unwrap_or_else(|| "default".to_string());
    if !valid_session(&session) {
        return Err(bad_request("`session` is a name of letters, digits, `-`, `_` and `.`, at most 64 characters".to_string()));
    }
    let yml = prepare(&request.yml, config, &session).map_err(|errors| failure(StatusCode::UNPROCESSABLE_ENTITY, &errors))?;
    std::fs::create_dir_all(config.data_dir.join(&session))
        .map_err(|e| failure(StatusCode::INTERNAL_SERVER_ERROR, &[Error::new(Phase::Execution, Path::root(), Rule::Io, format!("cannot create the folder of the session: {e}"))]))?;
    // the program is built again for each request: requests share nothing
    let program = build(&yml).map_err(|errors| failure(StatusCode::UNPROCESSABLE_ENTITY, &errors.0))?;
    Ok((program, request.message))
}

/// The answer that refuses a request when every permit to run is taken (503).
fn busy(max_runs: usize) -> Response {
    let message = format!("the service is running its most runs (max_runs: {max_runs}): try again later");
    failure(StatusCode::SERVICE_UNAVAILABLE, &[Error::new(Phase::Execution, Path::root(), Rule::TooManyRuns, message)])
}

async fn run(State(app): State<Arc<App>>, body: axum::body::Bytes) -> Response {
    let (program, message) = match accept(&app.config, &body) {
        Ok(x) => x,
        Err(refusal) => return refusal,
    };
    // taken without waiting, held until the run ends
    let Ok(permit) = app.permits.clone().try_acquire_owned() else { return busy(app.config.max_runs) };
    // the run is a task of its own that holds the permit: a caller that hangs up drops this handler, not the
    // run, so the permit is given back only when the run really ends (or is stopped by its timeout)
    let timeout = Duration::from_secs(app.config.run_timeout_seconds);
    let task = tokio::spawn(async move {
        let _permit = permit;
        run_within(program, &message, timeout).await
    });
    let outcome = task.await.unwrap_or_else(|e| Err(Error::new(Phase::Execution, Path::root(), Rule::Io, format!("the run stopped: {e}"))));
    match outcome {
        Ok(output) => Json200(output.to_json()).into_response(),
        Err(e) => failure(StatusCode::BAD_GATEWAY, &[e]),
    }
}

async fn start(State(app): State<Arc<App>>, body: axum::body::Bytes) -> Response {
    let (program, message) = match accept(&app.config, &body) {
        Ok(x) => x,
        Err(refusal) => return refusal,
    };
    let Ok(permit) = app.permits.clone().try_acquire_owned() else { return busy(app.config.max_runs) };
    let id = app.runs.start(program, message, permit);
    (StatusCode::ACCEPTED, axum::Json(json!({ "run": id }))).into_response()
}

fn not_found(id: &str) -> Response {
    failure(StatusCode::NOT_FOUND, &[Error::new(Phase::Execution, Path::root(), Rule::UnknownRef, format!("there is no run `{id}` (it never existed, or it ended long ago)"))])
}

async fn state(State(app): State<Arc<App>>, UrlPath(id): UrlPath<String>) -> Response {
    match app.runs.get(&id) {
        Some(run) => Json200(run.status()).into_response(),
        None => not_found(&id),
    }
}

/// The events already happened, then the new ones as they come, and at the end `result` or `error`.
async fn events(State(app): State<Arc<App>>, UrlPath(id): UrlPath<String>) -> Response {
    let Some(run) = app.runs.get(&id) else { return not_found(&id) };
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
                    let event: Result<SseEvent, Infallible> = Ok(SseEvent::default().event(name).data(data.to_string()));
                    return Some((event, (run, index + 1, false)));
                }
                Some(Next::Dropped(count)) => {
                    // the watcher came after these were dropped: it is told how many, and goes on from the first kept
                    let event: Result<SseEvent, Infallible> = Ok(SseEvent::default().event("dropped").data(json!({ "count": count }).to_string()));
                    return Some((event, (run, index + count, false)));
                }
                Some(Next::Last(name, data)) => {
                    let event: Result<SseEvent, Infallible> = Ok(SseEvent::default().event(name).data(data.to_string()));
                    return Some((event, (run, index, true)));
                }
                _ => {}
            }
        }
    });
    Sse::new(stream).keep_alive(KeepAlive::default()).into_response()
}
