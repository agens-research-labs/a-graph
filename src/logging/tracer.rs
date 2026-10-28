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
    name: String,
    start: u64,
    end: u64,
    attributes: Vec<(String, Value)>,
    /// Why the interval ended badly, if it did.
    error: Option<String>,
}

#[derive(Default)]
struct State {
    open: HashMap<u64, Span>,
    done: Vec<Span>,
}

pub struct Tracer {
    client: reqwest::Client,
    url: String,
    headers: reqwest::header::HeaderMap,
    delivery: Delivery,
    redaction: Redaction,
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
        let Some(mut t) = Obj::new(trace, base.clone(), &mut errs) else { return Err(Errors(errs.0)) };
        let url = t.req::<String>(key::ENDPOINT, &mut errs);
        let mut map = reqwest::header::HeaderMap::new();
        for (name, var) in t.secrets(key::HEADERS, &mut errs) {
            match std::env::var(&var) {
                Ok(value) => match (reqwest::header::HeaderName::from_bytes(name.as_bytes()), reqwest::header::HeaderValue::from_str(&value)) {
                    (Ok(n), Ok(v)) => {
                        map.insert(n, v);
                    }
                    _ => errs.add(&base.key(key::HEADERS).key(&name), Rule::BadValue, format!("`{name}` is not a valid header")),
                },
                Err(_) => errs.add(&base.key(key::HEADERS).key(&name), Rule::EnvMissing, format!("environment variable `{var}` is not set")),
            }
        }
        map.insert(reqwest::header::CONTENT_TYPE, reqwest::header::HeaderValue::from_static("application/json"));
        let delivery = Delivery::default();
        let client = match crate::net::client().timeout(delivery.timeout).build() {
            Ok(c) => Some(c),
            Err(e) => {
                errs.add(&base, Rule::Io, format!("cannot build the HTTP client: {e}"));
                None
            }
        };
        let service = passport_name(passport).unwrap_or_else(|| "a-graph".to_string());
        match (url, client, errs.0.is_empty()) {
            (Some(url), Some(client), true) => Ok(Some(Tracer { client, url, headers: map, delivery, redaction, service, state: Mutex::new(State::default()) })),
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
        let number = |k: &str| ev.fields.iter().find(|(n, _)| n == k).and_then(|(_, v)| v.as_f64());
        let mut attributes: Vec<(String, Value)> = ev.fields.iter().map(|(k, v)| (format!("a_graph.{k}"), v.clone())).collect();
        if let Some(n) = &ev.node {
            attributes.push(("a_graph.node".into(), json!(n)));
        }
        if !ev.content.is_empty() {
            if self.redaction.allow_content {
                attributes.extend(ev.content.iter().map(|(k, v)| (format!("a_graph.{k}"), json!(v))));
            } else {
                attributes.push(("a_graph.content_redacted".into(), json!(true)));
            }
        }
        let text = |k: &str| ev.fields.iter().find(|(n, _)| n == k).and_then(|(_, v)| v.as_str()).unwrap_or_default().to_string();
        match ev.code.as_str() {
            "run.start" => open(&mut st, ev, "run".into(), attributes),
            "group.start" => open(&mut st, ev, text("pattern"), attributes),
            "node.start" => open(&mut st, ev, format!("node {}", ev.node.clone().unwrap_or_default()), attributes),
            "run.end" | "group.end" | "node.end" => {
                if let Some(mut s) = st.open.remove(&ev.span) {
                    s.end = ev.at;
                    s.attributes.extend(attributes);
                    st.done.push(s);
                }
            }
            "model.call" => point(&mut st, ev, format!("model {}", text("model")), attributes, number("duration")),
            "tool.call" => {
                let error = ev.fields.iter().any(|(k, v)| k == "is_error" && v == &Value::Bool(true)).then(|| "the tool answered with an error".to_string());
                point(&mut st, ev, format!("{} {}", text("kind"), text("name")), attributes, number("duration"));
                if let (Some(e), Some(last)) = (error, st.done.last_mut()) {
                    last.error = Some(e);
                }
            }
            "read.vectordb" => point(&mut st, ev, format!("read vectordb {}", text("id")), attributes, number("duration")),
            "read.memory" => point(&mut st, ev, format!("read memory {}", text("id")), attributes, number("duration")),
            // an error of the run ends its interval, marked as failed
            _ if ev.level == LogLevel::Error => {
                if let Some(mut s) = st.open.remove(&ev.span) {
                    s.end = ev.at;
                    s.error = Some(format!("{}: {}", ev.code, text("message")));
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
        let mut spans = {
            let mut st = self.state.lock().unwrap();
            let open: Vec<Span> = st.open.drain().map(|(_, mut s)| {
                s.end = now;
                s.error.get_or_insert_with(|| "interrupted".to_string());
                s
            }).collect();
            st.done.extend(open);
            std::mem::take(&mut st.done)
        };
        if spans.is_empty() {
            return;
        }
        spans.sort_by_key(|s| (s.start, s.id));
        let trace_id = format!("{:032x}", uuid::Uuid::new_v4().as_u128());
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
                    "kind": 1,
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
        json!({ "resourceSpans": [{
            "resource": { "attributes": [{ "key": "service.name", "value": { "stringValue": self.service } }] },
            "scopeSpans": [{ "scope": { "name": "a-graph" }, "spans": spans }],
        }] })
    }
}

fn open(st: &mut State, ev: &Event, name: String, attributes: Vec<(String, Value)>) {
    st.open.insert(ev.span, Span { id: ev.span, parent: ev.parent, name, start: ev.at, end: ev.at, attributes, error: None });
}

fn point(st: &mut State, ev: &Event, name: String, attributes: Vec<(String, Value)>, duration: Option<f64>) {
    let took = (duration.unwrap_or(0.0).max(0.0) * 1e9) as u64;
    st.done.push(Span { id: ev.span, parent: ev.parent, name, start: ev.at.saturating_sub(took), end: ev.at, attributes, error: None });
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
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0)
}

/// `passport.adl.name`.
fn passport_name(passport: Option<Yml>) -> Option<String> {
    let mut errs = Errs::default();
    let mut p = Obj::new(passport?, Path::root().key(key::PASSPORT), &mut errs)?;
    p.sub(key::ADL, &mut errs)?.opt::<String>(key::NAME, &mut errs)
}
