//! The domain event: emitted once by the runtime, read by the consumers (logger, tracer,
//! verbose reporter). Plain data; the first consumer is `log::Logger`.

use serde_json::Value;

use super::LogLevel;
use crate::error::{Error, Warning};

#[derive(Debug, Clone)]
pub struct Event {
    pub level: LogLevel,
    /// Stable name of what happened, e.g. a `Rule` code.
    pub code: String,
    pub node: Option<String>,
    /// Safe to show whatever the data classification is: node, model, tokens, duration, outcome.
    pub fields: Vec<(String, Value)>,
    /// Message contents (prompts, responses): dropped by the redaction unless the passport allows them.
    pub content: Vec<(String, String)>,
    /// How many groups the event is inside of; set when the runtime emits it.
    pub depth: usize,
    /// The interval of a trace this event opens, closes or is (0: none), and the one around it.
    pub span: u64,
    pub parent: u64,
    /// When the event happened, in nanoseconds since 1970; set when the runtime emits it.
    pub at: u64,
}

impl Event {
    pub fn new(level: LogLevel, code: impl Into<String>) -> Self {
        Event {
            level,
            code: code.into(),
            node: None,
            fields: Vec::new(),
            content: Vec::new(),
            depth: 0,
            span: 0,
            parent: 0,
            at: 0,
        }
    }

    pub fn span(mut self, span: u64, parent: u64) -> Self {
        self.span = span;
        self.parent = parent;
        self
    }

    pub fn node(mut self, id: impl Into<String>) -> Self {
        self.node = Some(id.into());
        self
    }

    pub fn field(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.fields.push((key.into(), value.into()));
        self
    }

    pub fn content(mut self, key: impl Into<String>, text: impl Into<String>) -> Self {
        self.content.push((key.into(), text.into()));
        self
    }
}

impl From<&Error> for Event {
    fn from(e: &Error) -> Self {
        let ev = Event::new(LogLevel::Error, e.rule.code())
            .field("phase", e.phase.to_string())
            .field("path", e.path.to_string())
            .field("message", e.message.clone());
        match &e.detail {
            Some(detail) => ev.content("detail", detail.clone()),
            None => ev,
        }
    }
}

impl From<&Warning> for Event {
    fn from(w: &Warning) -> Self {
        Event::new(LogLevel::Warn, w.code)
            .field("phase", w.phase.to_string())
            .field("path", w.path.to_string())
            .field("message", w.message.clone())
    }
}
