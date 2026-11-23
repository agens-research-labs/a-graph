//! The events a run emits. Each goes to the logger and to the verbose reporter; the safe facts are
//! `fields`, the message contents are `content` (hidden unless the passport allows them).

use std::time::Instant;

use serde_json::Value;

use super::Program;
use crate::graph::Entity;
use crate::logging::{Event, LogLevel, TraceContext};
use crate::models::{Llm, ModelError, ModelInvocation, ModelResponse};

/// Where the code that runs is: how many groups it is inside of, and the interval of the trace
/// of the innermost one (0 = the run itself).
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Where {
    pub(super) depth: usize,
    pub(super) span: u64,
}

tokio::task_local! {
    pub(super) static CTX: Where;
}

pub(super) fn here() -> Where {
    CTX.try_with(|c| *c).unwrap_or_default()
}

/// The numbers of the intervals of a trace, and which one each node that is running has.
#[derive(Default)]
pub(crate) struct Spans {
    trace_id: std::sync::Mutex<Option<String>>,
    run: std::sync::atomic::AtomicU64,
    nodes: std::sync::Mutex<std::collections::HashMap<String, u64>>,
    /// The trace the next run goes in, chosen before it starts (`join_trace`); taken by that run only.
    joined: std::sync::Mutex<Option<Joined>>,
    /// Whether the trace of the run in progress is thrown away instead of sent: its caller does not record it.
    unsampled: std::sync::atomic::AtomicBool,
}

/// The trace and the span of a run, chosen before it starts.
struct Joined {
    trace_id: String,
    run: u64,
    /// The span of the caller, or 0.
    parent: u64,
    sampled: bool,
}

/// A new trace id: 32 hex digits of a random uuid.
fn new_trace_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

impl Spans {
    /// A new interval: random, so that two runs in the same trace never share one. It is the lower half of a random
    /// uuid, 62 random bits whose first bit is always 1: never 0, which means "none".
    fn id(&self) -> u64 {
        uuid::Uuid::new_v4().as_u64_pair().1
    }
}

/// The text of the reasoning blocks of a model: whatever they hold under `thinking` or `text`.
fn reasoning_text(blocks: &[Value]) -> String {
    fn collect(v: &Value, out: &mut Vec<String>) {
        match v {
            Value::Object(m) => {
                for (k, v) in m {
                    match (k.as_str(), v) {
                        ("thinking" | "text", Value::String(s)) if !s.trim().is_empty() => out.push(s.clone()),
                        _ => collect(v, out),
                    }
                }
            }
            Value::Array(a) => a.iter().for_each(|x| collect(x, out)),
            _ => {}
        }
    }
    let mut parts = Vec::new();
    blocks.iter().for_each(|b| collect(b, &mut parts));
    parts.join(" ")
}

impl Program {
    /// Sends an event to the logger, to the verbose reporter and to the tracer.
    pub(crate) fn emit(&self, mut ev: Event) {
        ev.depth = here().depth;
        ev.at = crate::logging::now_nanos();
        ev.trace_id = self.spans.trace_id.lock().unwrap().clone();
        if ev.span == 0 {
            ev.span = self.parent_of(ev.node.as_deref());
        }
        self.logger.log(&ev);
        self.verbose.show(&ev);
        if let Some(t) = &self.tracer {
            t.record(&ev);
        }
        if let Some(o) = &self.observer {
            o(&ev);
        }
    }

    /// The interval a new one belongs to: the node that is running, or the group, or the run.
    fn parent_of(&self, node: Option<&str>) -> u64 {
        let in_node = node.and_then(|n| self.spans.nodes.lock().unwrap().get(n).copied());
        match (in_node, here().span) {
            (Some(n), _) => n,
            (None, 0) => self.spans.run.load(std::sync::atomic::Ordering::SeqCst),
            (None, g) => g,
        }
    }

    /// Gives the event an interval of its own, inside the node it is about (a call, a reading).
    pub(super) fn traced(&self, ev: Event) -> Event {
        let parent = self.parent_of(ev.node.as_deref());
        let id = self.spans.id();
        ev.span(id, parent)
    }

    /// Puts the next run in the trace of the caller, or in a trace of its own when there is none, and gives the
    /// `traceparent` of that run: its trace, its span, and whether its trace is sent (a `trace` block, and a caller
    /// that records). Only the next run: the one after gets a new trace, as always.
    pub(crate) fn join_trace(&self, caller: Option<TraceContext>) -> String {
        let run = self.spans.id();
        let (trace_id, parent, sampled) = match caller {
            Some(c) => (c.trace_id, c.parent, c.sampled),
            None => (new_trace_id(), 0, true),
        };
        let traceparent = TraceContext::traceparent(&trace_id, run, sampled && self.tracer.is_some());
        *self.spans.joined.lock().unwrap() = Some(Joined {
            trace_id,
            run,
            parent,
            sampled,
        });
        traceparent
    }

    pub(super) fn run_started(&self) {
        let joined = self.spans.joined.lock().unwrap().take().unwrap_or_else(|| Joined {
            trace_id: new_trace_id(),
            run: self.spans.id(),
            parent: 0,
            sampled: true,
        });
        *self.spans.trace_id.lock().unwrap() = Some(joined.trace_id);
        self.spans.run.store(joined.run, std::sync::atomic::Ordering::SeqCst);
        self.spans
            .unsampled
            .store(!joined.sampled, std::sync::atomic::Ordering::SeqCst);
        // the parent of the run is the span of its caller, if any: inside the run nothing changes
        self.emit(Event::new(LogLevel::Info, "run.start").span(joined.run, joined.parent));
    }

    /// Sends the timeline of the run to the collector, whatever the outcome; or forgets it, when the caller of the
    /// run does not record its trace.
    pub(super) async fn send_trace(&self) {
        if let Some(t) = &self.tracer {
            if self.spans.unsampled.load(std::sync::atomic::Ordering::SeqCst) {
                t.discard();
            } else {
                t.flush().await;
            }
        }
    }

    pub(super) fn run_finished(&self, started: Instant, failure: Option<&crate::error::Error>) {
        let id = self.spans.run.load(std::sync::atomic::Ordering::SeqCst);
        match failure {
            None => self.emit(
                Event::new(LogLevel::Info, "run.end")
                    .span(id, 0)
                    .field("duration", started.elapsed().as_secs_f64()),
            ),
            Some(e) => self.emit(Event::from(e).span(id, 0)),
        }
    }

    pub(super) fn node_started(&self, node: Entity) {
        let model = self.nodes.model().get(node).map(|m| m.id.clone()).unwrap_or_default();
        let name = self.nodes.id(node).unwrap_or_default();
        let parent = self.parent_of(None);
        let id = self.spans.id();
        self.spans.nodes.lock().unwrap().insert(name.to_string(), id);
        self.emit(
            Event::new(LogLevel::Info, "node.start")
                .node(name)
                .field("model", model)
                .span(id, parent),
        );
    }

    pub(super) fn node_finished(&self, node: Entity, response: &ModelResponse, verdict: Option<&str>) {
        let model = self.nodes.model().get(node).map(|m| m.id.clone()).unwrap_or_default();
        let name = self.nodes.id(node).unwrap_or_default();
        let id = self.spans.nodes.lock().unwrap().remove(name).unwrap_or(0);
        let mut ev = Event::new(LogLevel::Info, "node.end")
            .span(id, 0)
            .node(name)
            .field("model", model)
            .field("input_tokens", response.info.usage.input_tokens)
            .field("output_tokens", response.info.usage.output_tokens)
            .field("duration", response.info.execution_time)
            .field("stop", format!("{:?}", response.stop_reason));
        if let Some(v) = verdict {
            ev = ev.field("verdict", v);
        }
        // a node whose answer was checked against a schema says how many corrections it needed
        if let Some(c) = response.info.corrections {
            ev = ev.field("corrections", c);
        }
        self.emit(ev);
    }

    /// Trace a provider-independent invocation, including failure and cancellation (an open span at flush).
    pub(super) async fn invoke_model(
        &self,
        node: &str,
        model: &str,
        llm: &dyn Llm,
        call: &ModelInvocation,
    ) -> Result<ModelResponse, ModelError> {
        let mut start = self.traced(
            Event::new(LogLevel::Debug, "model.start")
                .node(node)
                .field("model", model),
        );
        if let Some(spec) = self.models.spec(model) {
            start = start
                .field("gen_ai.request.model", spec.common().model.clone())
                .field("gen_ai.provider.name", spec.provider_name());
            if let Some((server, port)) = spec.server() {
                start = start.field("server.address", server).field("server.port", port);
            }
        }
        start = start.field("gen_ai.operation.name", "chat");
        let (span, parent) = (start.span, start.parent);
        self.emit(start);
        let began = Instant::now();
        let (result, http) = crate::models::http_telemetry::observe(llm.invoke(call)).await;
        let mut event = match &result {
            Ok(response) => self.model_called(node, model, call, response),
            Err(error) => {
                let (provider, kind, status) = match error {
                    ModelError::Unsupported { provider, .. } => (provider, "unsupported", None),
                    ModelError::Credentials { provider, .. } => (provider, "credentials", None),
                    ModelError::Network { provider, .. } => (provider, "network", None),
                    ModelError::Provider { provider, status, .. } => (provider, "provider", *status),
                    ModelError::InvalidResponse { provider, .. } => (provider, "invalid_response", None),
                    ModelError::Truncated { provider, .. } => (provider, "truncated", None),
                    ModelError::MimeMismatch { provider, .. } => (provider, "mime_mismatch", None),
                };
                let mut ev = Event::new(LogLevel::Error, "model.error")
                    .node(node)
                    .span(span, parent)
                    .field("model", model)
                    .field("gen_ai.provider.name", provider.clone())
                    .field("error.type", kind)
                    .field("message", "model invocation failed")
                    .field("duration", began.elapsed().as_secs_f64())
                    .content("system_prompt", call.system.clone())
                    .content("prompt", call.user.clone());
                if let Some(status) = status {
                    ev = ev.field("http.response.status_code", status);
                }
                if let Some(detail) = error.said() {
                    ev = ev.content("detail", detail);
                }
                ev
            }
        };
        for (key, value) in http.fields() {
            if let Some((_, old)) = event.fields.iter_mut().find(|(existing, _)| existing == &key) {
                *old = value;
            } else {
                event = event.field(key, value);
            }
        }
        self.emit(event.span(span, parent));
        result
    }

    /// One successful call, with the text it got and the text it gave.
    fn model_called(&self, node: &str, model: &str, call: &ModelInvocation, response: &ModelResponse) -> Event {
        let answer = response
            .message
            .clone()
            .or_else(|| response.structured.as_ref().map(Value::to_string))
            .unwrap_or_default();
        let mut ev = Event::new(LogLevel::Debug, "model.call")
            .node(node)
            .field("model", model)
            .field("gen_ai.provider.name", response.info.provider.clone())
            .field("gen_ai.response.model", response.info.model.clone())
            .field("gen_ai.operation.name", "chat")
            .field(
                "gen_ai.usage.input_tokens",
                response.info.usage.input_tokens
                    + response.info.usage.cache_read_tokens
                    + response.info.usage.cache_write_tokens,
            )
            .field("gen_ai.usage.output_tokens", response.info.usage.output_tokens)
            .field("input_tokens", response.info.usage.input_tokens)
            .field("output_tokens", response.info.usage.output_tokens)
            .field("duration", response.info.execution_time)
            .content("system_prompt", call.system.clone())
            .content("prompt", call.user.clone())
            .content("answer", answer);
        let reasoning = reasoning_text(&response.reasoning);
        if !reasoning.is_empty() {
            ev = ev.content("reasoning", reasoning);
        }
        ev
    }

    /// A group begins: its interval, and when it began.
    pub(super) fn group_started(&self, pattern: &str, branches: usize) -> (Instant, u64) {
        let parent = self.parent_of(None);
        let id = self.spans.id();
        self.emit(
            Event::new(LogLevel::Info, "group.start")
                .field("pattern", pattern)
                .field("branches", branches)
                .span(id, parent),
        );
        (Instant::now(), id)
    }

    pub(super) fn group_finished(&self, pattern: &str, started: Instant, span: u64) {
        self.emit(
            Event::new(LogLevel::Info, "group.end")
                .field("pattern", pattern)
                .field("duration", started.elapsed().as_secs_f64())
                .span(span, 0),
        );
    }
}
