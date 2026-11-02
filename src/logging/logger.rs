//! The logger: consumer of the event stream, configured by the validated `observability` block.
//! One JSON object per line. Redaction is applied here, before anything is written.

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::sync::Mutex;

use serde_json::{Map, Value, json};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tokio::task::JoinHandle;

use super::delivery::{Delivery, deliver};
use super::{Event, LogLevel, Redaction};
use crate::build::{Errs, Obj};
use crate::error::{Error, Errors, Path, Phase, Rule};
use crate::schema::key;

pub struct Logger {
    level: LogLevel,
    redaction: Redaction,
    out: Mutex<Box<dyn Write + Send>>,
    /// Background task of a network output; `close` waits for it.
    task: Option<JoinHandle<()>>,
}

impl Logger {
    pub fn new(level: LogLevel, redaction: Redaction, out: Box<dyn Write + Send>) -> Self {
        Logger {
            level,
            redaction,
            out: Mutex::new(out),
            task: None,
        }
    }

    /// Sends the events to `url` with the default `Delivery`. See `to_url_with`.
    pub fn to_url(
        level: LogLevel,
        redaction: Redaction,
        url: &str,
        headers: Vec<(String, String)>,
    ) -> Result<Self, Error> {
        Self::to_url_with(level, redaction, url, headers, Delivery::default())
    }

    /// Sends the events as `application/x-ndjson` POSTs from a background task.
    /// Needs a running tokio runtime; call `close` before exit to deliver what is queued.
    pub fn to_url_with(
        level: LogLevel,
        redaction: Redaction,
        url: &str,
        headers: Vec<(String, String)>,
        delivery: Delivery,
    ) -> Result<Self, Error> {
        let at = Path::root().key("observability").key("output");
        let fail = |key: &str, rule: Rule, msg: String| Error::new(Phase::Interpretation, at.key(key), rule, msg);
        let handle = tokio::runtime::Handle::try_current().map_err(|_| {
            fail(
                "url",
                Rule::NotSupported,
                "logging to a network endpoint needs a running tokio runtime".into(),
            )
        })?;
        let mut map = reqwest::header::HeaderMap::new();
        for (name, value) in headers {
            let n = reqwest::header::HeaderName::from_bytes(name.as_bytes())
                .map_err(|_| fail("headers", Rule::BadValue, format!("invalid header name `{name}`")))?;
            let v = reqwest::header::HeaderValue::from_str(&value)
                .map_err(|_| fail("headers", Rule::BadValue, format!("invalid value for header `{name}`")))?;
            map.insert(n, v);
        }
        map.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/x-ndjson"),
        );
        let client = crate::net::client()
            .timeout(delivery.timeout)
            .build()
            .map_err(|e| fail("url", Rule::Io, format!("cannot build the HTTP client: {e}")))?;

        let url = url.to_string();
        let (tx, mut rx) = unbounded_channel::<String>();
        let task = handle.spawn(async move {
            while let Some(first) = rx.recv().await {
                // everything already queued goes in the same POST, in order
                let mut body = first + "\n";
                let mut n = 1;
                while n < delivery.max_batch {
                    let Ok(line) = rx.try_recv() else { break };
                    body.push_str(&line);
                    body.push('\n');
                    n += 1;
                }
                deliver(&client, &url, &map, body, &delivery, "log").await;
            }
        });
        let mut logger = Logger::new(level, redaction, Box::new(ChannelWriter { tx, buf: Vec::new() }));
        logger.task = Some(task);
        Ok(logger)
    }

    /// Stops accepting events and waits until the queued ones are delivered.
    pub async fn close(self) {
        let Logger { out, task, .. } = self;
        drop(out);
        if let Some(t) = task {
            let _ = t.await;
        }
    }

    /// Builds the logger from the `passport` and `observability` blocks, already validated by the
    /// Loader. `{ env }` header values are read here: a variable not set is an error.
    pub fn build(
        passport: Option<serde_norway::Value>,
        observability: Option<serde_norway::Value>,
    ) -> Result<Self, Errors> {
        let redaction = Redaction::from_passport(passport)?;
        let mut errs = Errs::default();
        let base = Path::root().key(key::OBSERVABILITY);
        let Some(mut obs) = observability.and_then(|v| Obj::new(v, base.clone(), &mut errs)) else {
            errs.add(&base, Rule::MissingField, "`observability` is mandatory");
            return Err(Errors(errs.0));
        };
        let level = obs.req::<LogLevel>(key::LEVEL, &mut errs);
        let output = obs.value(key::OUTPUT);
        let at = base.key(key::OUTPUT);
        let logger = match (level, output) {
            (Some(level), None) => Some(Logger::new(level, redaction, Box::new(io::stderr()))),
            (Some(level), Some(output)) => Self::with_output(level, redaction, output, &at, &mut errs),
            (None, _) => None,
        };
        match logger {
            Some(l) if errs.0.is_empty() => Ok(l),
            _ => Err(Errors(errs.0)),
        }
    }

    /// `stdout`, `{ file }` or `{ url, headers }`.
    fn with_output(
        level: LogLevel,
        redaction: Redaction,
        output: serde_norway::Value,
        at: &Path,
        errs: &mut Errs,
    ) -> Option<Self> {
        match output.as_str() {
            Some("stdout") => return Some(Logger::new(level, redaction, Box::new(io::stdout()))),
            Some("stderr") => return Some(Logger::new(level, redaction, Box::new(io::stderr()))),
            _ => {}
        }
        let mut o = Obj::new(output, at.clone(), errs)?;
        if let Some(file) = o.opt::<String>(key::FILE, errs) {
            return match OpenOptions::new().create(true).append(true).open(&file) {
                Ok(f) => Some(Logger::new(level, redaction, Box::new(f))),
                Err(e) => {
                    errs.add(&at.key(key::FILE), Rule::Io, format!("cannot open `{file}`: {e}"));
                    None
                }
            };
        }
        let url = o.req::<String>(key::URL, errs)?;
        let mut headers = Vec::new();
        for (name, var) in o.secrets(key::HEADERS, errs) {
            match std::env::var(&var) {
                Ok(value) => headers.push((name, value)),
                Err(_) => errs.add(
                    &at.key(key::HEADERS).key(&name),
                    Rule::EnvMissing,
                    format!("environment variable `{var}` is not set"),
                ),
            }
        }
        if !errs.0.is_empty() {
            return None;
        }
        match Logger::to_url(level, redaction, &url, headers) {
            Ok(l) => Some(l),
            Err(e) => {
                errs.0.push(e);
                None
            }
        }
    }

    pub fn log(&self, ev: &Event) {
        if ev.level < self.level {
            return;
        }
        let mut line = Map::new();
        for (k, v) in &ev.fields {
            line.insert(k.clone(), v.clone());
        }
        // the fixed keys come last: an event field cannot overwrite them
        line.insert(
            "ts".into(),
            json!(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)),
        );
        line.insert("level".into(), json!(format!("{:?}", ev.level).to_lowercase()));
        line.insert("code".into(), json!(ev.code));
        if let Some(n) = &ev.node {
            line.insert("node".into(), json!(n));
        }
        if !ev.content.is_empty() {
            if self.redaction.allow_content {
                for (k, v) in &ev.content {
                    line.insert(k.clone(), json!(v));
                }
            } else {
                line.insert("content_redacted".into(), Value::Bool(true));
            }
        }
        let mut text = Value::Object(line).to_string();
        text.push('\n');
        // One write per line: the runs of a session append to the same file, each with its own logger, and a line
        // written in two parts could interleave with another. A failing log write must not stop the run: tell
        // stderr instead.
        if let Err(e) = self.out.lock().unwrap().write_all(text.as_bytes()) {
            eprintln!("a-graph: cannot write the log: {e}");
        }
    }

    pub fn log_error(&self, e: &Error) {
        self.log(&Event::from(e));
    }
}

/// Hands every complete line to the background task of a network output.
struct ChannelWriter {
    tx: UnboundedSender<String>,
    buf: Vec<u8>,
}

impl Write for ChannelWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.buf.extend_from_slice(bytes);
        while let Some(pos) = self.buf.iter().position(|b| *b == b'\n') {
            let line: Vec<u8> = self.buf.drain(..=pos).collect();
            let line = String::from_utf8_lossy(&line[..pos]).into_owned();
            self.tx
                .send(line)
                .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "the log task has ended"))?;
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
