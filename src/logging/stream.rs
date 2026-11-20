//! An event of a run as it is sent to someone who watches it live (the service streams them).

use serde_json::{Map, Value};

use super::{Event, LogLevel, VerboseLevel};

/// `(name, data)` of the event if `level` shows it, like the `verbose` display does: `steps` the run,
/// the nodes and the groups, `detail` also the calls and the readings. The contents of the messages
/// are in `data` only if the passport allows them.
pub fn stream_event(ev: &Event, level: VerboseLevel, allow_content: bool) -> Option<(String, Value)> {
    let needed = match ev.code.as_str() {
        "run.start" | "run.end" | "node.start" | "node.end" | "group.start" | "group.end" => VerboseLevel::Steps,
        "model.call" | "tool.call" | "utility.call" | "read.vectordb" | "read.memory" | "read.media"
        | "read.prompt" => VerboseLevel::Detail,
        _ if ev.level >= LogLevel::Warn => VerboseLevel::Steps,
        _ => return None,
    };
    if level == VerboseLevel::Off || level < needed {
        return None;
    }
    let mut data = Map::new();
    if let Some(node) = &ev.node {
        data.insert("node".into(), node.clone().into());
    }
    for (k, v) in &ev.fields {
        data.insert(k.clone(), v.clone());
    }
    if !ev.content.is_empty() {
        if allow_content {
            for (k, v) in &ev.content {
                data.insert(k.clone(), v.clone().into());
            }
        } else {
            data.insert("content_redacted".into(), Value::Bool(true));
        }
    }
    Some((ev.code.clone(), Value::Object(data)))
}
