//! The tracer: consumer of the event stream that builds the timeline of a run, one interval
//! (span) per run, group, node, call and reading, and sends it to an OpenTelemetry collector as
//! OTLP over HTTP in JSON when the run is over.

use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::{Value, json};
use serde_norway::Value as Yml;

use super::delivery::{Delivery, deliver};
use super::{Event, LogLevel, Redaction};
use crate::build::{Errs, Obj};
use crate::error::{Errors, Path, Rule};
use crate::schema::key;

/// An interval of the trace, open or closed.
struct Span {
    id: u64,
    parent: u64,
    kind: u8,
    name: String,
    start: u64,
    end: u64,
    attributes: Vec<(String, Value)>,
    /// Why the interval ended badly, if it did.
    error: Option<String>,
}

#[derive(Default)]
struct State {
    trace_id: Option<String>,
    open: HashMap<u64, Span>,
    done: Vec<Span>,
}

pub struct Tracer {
    client: reqwest::Client,
    url: String,
    headers: reqwest::header::HeaderMap,
    delivery: Delivery,
    redaction: Redaction,
    capture_content: bool,
    max_content_bytes: usize,
    graph_version: Option<String>,
    /// The name the collector shows for the service: the name of the passport.
    service: String,
    state: Mutex<State>,
}

impl Tracer {
    /// Builds from the `trace` block; `None` when the block is absent (trace off). `{ env }` header
    /// values are read here: a variable not set is an error.
    pub fn build(trace: Option<Yml>, passport: Option<Yml>, redaction: Redaction) -> Result<Option<Self>, Errors> {
        let Some(trace) = trace else { return Ok(None) };
        let mut errs = Errs::default();
        let base = Path::root().key(key::TRACE);
        let Some(mut t) = Obj::new(trace, base.clone(), &mut errs) else {
            return Err(Errors(errs.0));
        };
        let url = t.req::<String>(key::ENDPOINT, &mut errs);
        let mut map = reqwest::header::HeaderMap::new();
        for (name, var) in t.secrets(key::HEADERS, &mut errs) {
            match std::env::var(&var) {
                Ok(value) => match (
                    reqwest::header::HeaderName::from_bytes(name.as_bytes()),
                    crate::net::secret_value(&value),
                ) {
                    (Ok(n), Ok(v)) => {
                        map.insert(n, v);
                    }
                    _ => errs.add(
                        &base.key(key::HEADERS).key(&name),
                        Rule::BadValue,
                        format!("`{name}` is not a valid header"),
                    ),
                },
                Err(_) => errs.add(
                    &base.key(key::HEADERS).key(&name),
                    Rule::EnvMissing,
                    format!("environment variable `{var}` is not set"),
                ),
            }
        }
        map.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        let capture_content = t.opt::<bool>(key::CAPTURE_CONTENT, &mut errs).unwrap_or(false);
        let max_content_bytes = t.opt::<usize>(key::MAX_CONTENT_BYTES, &mut errs).unwrap_or(4096);
        if max_content_bytes == 0 {
            errs.add(&base.key(key::MAX_CONTENT_BYTES), Rule::BadValue, "must be at least 1");
        }
        let delivery = Delivery::default();
        let client = match crate::net::client().timeout(delivery.timeout).build() {
            Ok(c) => Some(c),
            Err(e) => {
                errs.add(&base, Rule::Io, format!("cannot build the HTTP client: {e}"));
                None
            }
        };
        let service = passport_field(passport.clone(), key::NAME).unwrap_or_else(|| "a-graph".to_string());
        let graph_version = passport_field(passport, key::VERSION);
        match (url, client, errs.0.is_empty()) {
            (Some(url), Some(client), true) => Ok(Some(Tracer {
                client,
                url,
                headers: map,
                delivery,
                redaction,
                capture_content,
                max_content_bytes,
                graph_version,
                service,
                state: Mutex::new(State::default()),
            })),
            _ => Err(Errors(errs.0)),
        }
    }

    /// Takes an event into the timeline: `*.start` opens an interval, `*.end` closes it, a call or a
    /// reading is an interval of its own that ends now and began `duration` ago.
    pub fn record(&self, ev: &Event) {
        if ev.span == 0 {
            return;
        }
        let mut st = self.state.lock().unwrap();
        if let Some(id) = &ev.trace_id {
            st.trace_id = Some(id.clone());
        }
        let number = |k: &str| ev.fields.iter().find(|(n, _)| n == k).and_then(|(_, v)| v.as_f64());
        let mut attributes: Vec<(String, Value)> = ev
            .fields
            .iter()
            .map(|(k, v)| {
                (
                    if k.starts_with("gen_ai.")
                        || k == "error.type"
                        || k.starts_with("http.response.")
                        || k.starts_with("http.request.")
                        || k.starts_with("server.")
                        || k == "url.scheme"
                    {
                        k.clone()
                    } else {
                        format!("a_graph.{k}")
                    },
                    v.clone(),
                )
            })
            .collect();
        if let Some(n) = &ev.node {
            attributes.push(("a_graph.node".into(), json!(n)));
        }
        let mut contents: Vec<(String, Value)> = Vec::new();
        if !ev.content.is_empty() {
            if self.redaction.allow_content && self.capture_content {
                let mut truncated = false;
                for (key, text) in &ev.content {
                    let mut end = text.len().min(self.max_content_bytes);
                    while !text.is_char_boundary(end) {
                        end -= 1;
                    }
                    truncated |= end < text.len();
                    contents.push((format!("a_graph.{key}"), json!(&text[..end])));
                }
                if truncated {
                    contents.push(("a_graph.content_truncated".into(), json!(true)));
                }
            } else {
                contents.push(("a_graph.content_redacted".into(), json!(true)));
            }
        }
        attributes.extend(contents.iter().cloned());
        let text = |k: &str| {
            ev.fields
                .iter()
                .find(|(n, _)| n == k)
                .and_then(|(_, v)| v.as_str())
                .unwrap_or_default()
                .to_string()
        };
        match ev.code.as_str() {
            "run.start" => open(&mut st, ev, "run".into(), attributes),
            "group.start" => open(&mut st, ev, text("pattern"), attributes),
            "model.start" => open(&mut st, ev, format!("model {}", text("model")), attributes),
            "node.start" => open(
                &mut st,
                ev,
                format!("node {}", ev.node.clone().unwrap_or_default()),
                attributes,
            ),
            "run.end" | "group.end" | "node.end" | "model.call" => {
                if let Some(mut s) = st.open.remove(&ev.span) {
                    s.end = ev.at;
                    merge_attributes(&mut s.attributes, attributes);
                    st.done.push(s);
                }
            }
            "tool.call" => {
                let error = ev
                    .fields
                    .iter()
                    .any(|(k, v)| k == "is_error" && v == &Value::Bool(true))
                    .then(|| "the tool answered with an error".to_string());
                point(
                    &mut st,
                    ev,
                    format!("{} {}", text("kind"), text("name")),
                    attributes,
                    number("duration"),
                );
                if let (Some(e), Some(last)) = (error, st.done.last_mut()) {
                    last.error = Some(e);
                }
            }
            "structured.attempt" => point(&mut st, ev, format!("structured {}", text("outcome")), attributes, None),
            "read.vectordb" => point(
                &mut st,
                ev,
                format!("read vectordb {}", text("id")),
                attributes,
                number("duration"),
            ),
            "read.memory" => point(
                &mut st,
                ev,
                format!("read memory {}", text("id")),
                attributes,
                number("duration"),
            ),
            // an error of the run ends its interval, marked as failed, with its detail as the passport allows
            _ if ev.level == LogLevel::Error => {
                if let Some(mut s) = st.open.remove(&ev.span) {
                    s.end = ev.at;
                    let message = format!("{}: {}", ev.code, text("message"));
                    s.error = Some(message.clone());
                    merge_attributes(&mut s.attributes, attributes);
                    let mut parent = s.parent;
                    while let Some(ancestor) = st.open.get_mut(&parent) {
                        ancestor.error = Some(message.clone());
                        parent = ancestor.parent;
                    }
                    st.done.push(s);
                }
            }
            _ => {}
        }
    }

    /// Sends the timeline of the run and starts a new one. The intervals still open (the run failed
    /// in the middle of them) end now, marked as interrupted. A failing send tells stderr and goes on.
    pub async fn flush(&self) {
        let now = now_nanos();
        let (trace_id, mut spans) = {
            let mut st = self.state.lock().unwrap();
            // Interrupted descendants must not extend past the already-closed run span.
            let ended = st
                .done
                .iter()
                .find(|s| s.parent == 0 && s.name == "run")
                .map(|s| s.end)
                .unwrap_or(now);
            let open: Vec<Span> = st
                .open
                .drain()
                .map(|(_, mut s)| {
                    s.end = ended.max(s.start);
                    s.error.get_or_insert_with(|| "interrupted".to_string());
                    if s.kind == 3 {
                        merge_attributes(&mut s.attributes, vec![("error.type".into(), json!("cancelled"))]);
                    }
                    s
                })
                .collect();
            st.done.extend(open);
            (
                st.trace_id
                    .take()
                    .unwrap_or_else(|| uuid::Uuid::new_v4().simple().to_string()),
                std::mem::take(&mut st.done),
            )
        };
        if spans.is_empty() {
            return;
        }
        spans.sort_by_key(|s| (s.start, s.id));
        let body = self.body(&trace_id, &spans).to_string();
        deliver(&self.client, &self.url, &self.headers, body, &self.delivery, "trace").await;
    }

    /// The OTLP/JSON document of one trace.
    fn body(&self, trace_id: &str, spans: &[Span]) -> Value {
        let spans: Vec<Value> = spans
            .iter()
            .map(|s| {
                let mut span = json!({
                    "traceId": trace_id,
                    "spanId": format!("{:016x}", s.id),
                    "name": s.name,
                    "kind": s.kind,
                    "startTimeUnixNano": s.start.to_string(),
                    "endTimeUnixNano": s.end.max(s.start).to_string(),
                    "attributes": s.attributes.iter().map(|(k, v)| json!({ "key": k, "value": any_value(v) })).collect::<Vec<_>>(),
                });
                if s.parent != 0 {
                    span["parentSpanId"] = json!(format!("{:016x}", s.parent));
                }
                if let Some(e) = &s.error {
                    span["status"] = json!({ "code": 2, "message": e });
                }
                span
            })
            .collect();
        let mut resource = vec![
            json!({ "key": "service.name", "value": { "stringValue": self.service } }),
            json!({ "key": "service.version", "value": { "stringValue": env!("CARGO_PKG_VERSION") } }),
        ];
        if let Some(version) = &self.graph_version {
            resource.push(json!({ "key": "a_graph.graph.version", "value": { "stringValue": version } }));
        }
        json!({ "resourceSpans": [{
            "resource": { "attributes": resource },
            "scopeSpans": [{ "scope": { "name": "a-graph", "version": env!("CARGO_PKG_VERSION") }, "spans": spans }],
        }] })
    }
}

fn open(st: &mut State, ev: &Event, name: String, attributes: Vec<(String, Value)>) {
    st.open.insert(
        ev.span,
        Span {
            id: ev.span,
            parent: ev.parent,
            kind: if ev.code == "model.start" { 3 } else { 1 },
            name,
            start: ev.at,
            end: ev.at,
            attributes,
            error: None,
        },
    );
}

fn point(st: &mut State, ev: &Event, name: String, attributes: Vec<(String, Value)>, duration: Option<f64>) {
    let took = (duration.unwrap_or(0.0).max(0.0) * 1e9) as u64;
    st.done.push(Span {
        id: ev.span,
        parent: ev.parent,
        kind: 1,
        name,
        start: ev.at.saturating_sub(took),
        end: ev.at,
        attributes,
        error: None,
    });
}

/// An OTLP attribute value.
fn any_value(v: &Value) -> Value {
    match v {
        Value::Bool(b) => json!({ "boolValue": b }),
        Value::Number(n) if n.is_i64() || n.is_u64() => json!({ "intValue": n.to_string() }),
        Value::Number(n) => json!({ "doubleValue": n }),
        Value::String(s) => json!({ "stringValue": s }),
        other => json!({ "stringValue": other.to_string() }),
    }
}

/// Nanoseconds since 1970.
pub fn now_nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

fn merge_attributes(target: &mut Vec<(String, Value)>, attributes: Vec<(String, Value)>) {
    for (key, value) in attributes {
        if let Some((_, old)) = target.iter_mut().find(|(k, _)| k == &key) {
            *old = value;
        } else {
            target.push((key, value));
        }
    }
}

/// An identity field of `passport.adl`.
fn passport_field(passport: Option<Yml>, field: &str) -> Option<String> {
    let mut errs = Errs::default();
    let mut p = Obj::new(passport?, Path::root().key(key::PASSPORT), &mut errs)?;
    p.sub(key::ADL, &mut errs)?.opt::<String>(field, &mut errs)
}
