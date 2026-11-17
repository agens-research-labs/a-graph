//! Reading the components of a node from the yml, with the names of `schema::key`.

use serde_norway::Value;

use super::component::*;
use crate::blackboards::Place;
use crate::build::{Errs, Obj};
use crate::error::Rule;
use crate::models::Reasoning;
use crate::schema::key;

/// The most alternatives of a token a node may ask: the limit of OpenAI, the only one written down.
const MAX_LOGPROBS: u32 = 20;

pub(super) fn model(mut o: Obj, errs: &mut Errs) -> Option<Model> {
    let id = o.req::<String>(key::ID, errs);
    Some(Model {
        temperature: o.opt(key::TEMPERATURE, errs),
        top_p: o.opt(key::TOP_P, errs),
        max_tokens: o.opt(key::MAX_TOKENS, errs),
        reasoning: o.opt::<Reasoning>(key::REASONING, errs),
        id: id?,
    })
}

fn query(mut o: Obj, errs: &mut Errs) -> Option<Retrieval> {
    let (id, q) = (o.req::<String>(key::ID, errs), o.req::<String>(key::QUERY, errs));
    let query = if q.as_deref()? == key::USER_MESSAGE {
        Query::UserMessage
    } else {
        Query::Static(q?)
    };
    let (mut query_utility, mut chunks_utility) = (Vec::new(), Vec::new());
    if let Some(mut u) = o.sub(key::UTILITY, errs) {
        query_utility = utility_ids_at(&mut u, key::QUERY, errs);
        chunks_utility = utility_ids_at(&mut u, key::CHUNKS, errs);
    }
    Some(Retrieval {
        id: id?,
        query,
        query_utility,
        chunks_utility,
    })
}

fn roles(mut o: Obj, errs: &mut Errs) -> Option<(Roles, Vec<String>)> {
    let utility = utility_ids(&mut o, errs);
    let (user, assistant) = (o.req::<bool>(key::USER, errs), o.req::<bool>(key::ASSISTANT, errs));
    Some((
        Roles {
            user: user?,
            assistant: assistant?,
        },
        utility,
    ))
}

fn memory(mut o: Obj, errs: &mut Errs) -> Option<MemoryUse> {
    let id = o.req::<String>(key::ID, errs);
    let read = o.req_sub(key::READ, errs).and_then(|r| roles(r, errs));
    let write = o.req_sub(key::WRITE, errs).and_then(|w| roles(w, errs));
    let ((read, read_utility), (write, write_utility)) = (read?, write?);
    Some(MemoryUse {
        id: id?,
        read,
        write,
        read_utility,
        write_utility,
    })
}

/// A list of maps under `name`, each read with `one`.
fn list<T>(o: &mut Obj, name: &str, errs: &mut Errs, one: fn(Obj, &mut Errs) -> Option<T>) -> Vec<T> {
    let Some(v) = o.value(name) else { return Vec::new() };
    Obj::items(&o.at().key(name), Some(v), errs)
        .into_iter()
        .filter_map(|item| one(item, errs))
        .collect()
}

/// A validator reads the whole conversation of a memory and never writes it.
fn judge_memory(mut o: Obj, errs: &mut Errs) -> Option<MemoryUse> {
    let id = o.req::<String>(key::ID, errs)?;
    Some(MemoryUse {
        id,
        read: Roles {
            user: true,
            assistant: true,
        },
        write: Roles {
            user: false,
            assistant: false,
        },
        read_utility: Vec::new(),
        write_utility: Vec::new(),
    })
}

pub(super) fn external(mut o: Obj, errs: &mut Errs, judge: bool) -> External {
    let memory = if judge { judge_memory } else { memory };
    External {
        vectordb: list(&mut o, key::VECTORDB, errs, query),
        search: list(&mut o, key::SEARCH, errs, query),
        memory: list(&mut o, key::MEMORY, errs, memory),
    }
}

pub(super) fn resources(mut o: Obj, errs: &mut Errs) -> Resources {
    let max_rounds = o.opt::<u32>(key::MAX_ROUNDS, errs);
    if max_rounds == Some(0) {
        errs.add(
            &o.at().key(key::MAX_ROUNDS),
            Rule::BadValue,
            "`max_rounds` is at least 1",
        );
    }
    Resources {
        max_rounds,
        prompt: o.opt(key::PROMPT, errs),
        tools: o.opt(key::TOOLS, errs).unwrap_or_default(),
        mcp_servers: o.opt(key::MCP_SERVERS, errs).unwrap_or_default(),
        skills: o.opt(key::SKILLS, errs).unwrap_or_default(),
    }
}

pub(super) fn blackboard(mut o: Obj, errs: &mut Errs) -> Option<Blackboard> {
    let id = o.req::<String>(key::ID, errs);
    let write = o.sub(key::WRITE, errs).and_then(|mut w| {
        let utility = utility_ids(&mut w, errs);
        let (enabled, place) = (w.req::<bool>(key::ENABLED, errs), w.req::<Place>(key::PLACE, errs));
        Some(BlackboardWrite {
            enabled: enabled?,
            place: place?,
            utility,
        })
    });
    Some(Blackboard {
        read: o.opt(key::READ, errs).unwrap_or(false),
        write,
        cleanup: o.opt(key::CLEANUP, errs).unwrap_or(false),
        id: id?,
    })
}

fn media_in(mut o: Obj, errs: &mut Errs) -> Option<MediaIn> {
    let utility = utility_ids(&mut o, errs);
    let (id, from) = (o.req::<String>(key::ID, errs), o.req::<String>(key::FROM, errs));
    let from = if from.as_deref()? == "node" {
        InFrom::Node
    } else {
        InFrom::Resources
    };
    Some(MediaIn { id: id?, from, utility })
}

fn media_out(mut o: Obj, errs: &mut Errs) -> Option<MediaOut> {
    let utility = utility_ids(&mut o, errs);
    let id = o.req::<String>(key::ID, errs);
    let from = if o.opt::<String>(key::FROM, errs).as_deref() == Some("message") {
        OutFrom::Message
    } else {
        OutFrom::Model
    };
    Some(MediaOut {
        id: id?,
        from,
        utility,
        assistant: o.opt(key::ASSISTANT, errs).unwrap_or(false),
        node: o.opt(key::NODE, errs).unwrap_or(false),
    })
}

pub(super) fn media(mut o: Obj, errs: &mut Errs) -> Media {
    Media {
        input: list(&mut o, key::INPUT, errs, media_in),
        output: list(&mut o, key::OUTPUT, errs, media_out),
    }
}

/// The ids of a `utility` key: one id, or a list of ids.
pub(super) fn utility_ids(o: &mut Obj, errs: &mut Errs) -> Vec<String> {
    utility_ids_at(o, key::UTILITY, errs)
}

/// Like `utility_ids`, under another key (`query`, `chunks`).
pub(super) fn utility_ids_at(o: &mut Obj, name: &str, errs: &mut Errs) -> Vec<String> {
    match o.value(name) {
        None => Vec::new(),
        Some(Value::String(id)) => vec![id],
        Some(list) => serde_norway::from_value(list).unwrap_or_else(|e| {
            errs.add(&o.at().key(name), Rule::BadValue, e.to_string());
            Vec::new()
        }),
    }
}

fn message_input(mut o: Obj, errs: &mut Errs) -> (MessageInput, Vec<String>) {
    let utility = utility_ids(&mut o, errs);
    (
        MessageInput {
            user_message: o.opt(key::USER_MESSAGE, errs).unwrap_or(false),
            agent_message: o.opt(key::AGENT_MESSAGE, errs).unwrap_or(false),
        },
        utility,
    )
}

pub(super) fn message(mut o: Obj, errs: &mut Errs) -> Message {
    let (input, input_utility) = match o.sub(key::INPUT, errs).map(|i| message_input(i, errs)) {
        Some((input, utility)) => (Some(input), utility),
        None => (None, Vec::new()),
    };
    let output = o.sub(key::OUTPUT, errs).map(|mut p| {
        let at = p.at().key(key::SCHEMA);
        let utility = utility_ids(&mut p, errs);
        let logprobs = p.opt::<u32>(key::LOGPROBS, errs);
        if logprobs.is_some_and(|n| n > MAX_LOGPROBS) {
            errs.add(&p.at().key(key::LOGPROBS), Rule::BadValue, "logprobs is 0-20");
        }
        MessageOutput {
            utility,
            assistant_message: p.opt(key::ASSISTANT_MESSAGE, errs).unwrap_or(false),
            node: p.opt(key::NODE, errs).unwrap_or(false),
            reasoning: p.opt(key::REASONING, errs).unwrap_or(false),
            logprobs,
            schema: p.value(key::SCHEMA).and_then(|v| schema(v, &at, errs)),
        }
    });
    Message {
        input,
        output,
        input_utility,
    }
}

/// A JSON Schema written as a JSON string or as yml: a-graph checks only that it is a JSON object.
fn schema(v: Value, at: &crate::error::Path, errs: &mut Errs) -> Option<serde_json::Value> {
    let json = match v {
        Value::String(s) => {
            serde_json::from_str::<serde_json::Value>(&s).map_err(|e| format!("the schema is not valid JSON: {e}"))
        }
        other => serde_json::to_value(other).map_err(|e| format!("the schema cannot be read as JSON: {e}")),
    };
    match json {
        Ok(j) if j.is_object() => Some(j),
        Ok(_) => {
            errs.add(at, Rule::BadValue, "the schema must be a JSON object");
            None
        }
        Err(why) => {
            errs.add(at, Rule::BadValue, why);
            None
        }
    }
}

pub(super) fn validation(mut o: Obj, errs: &mut Errs) -> Option<Validation> {
    let on_error = o
        .req::<String>(key::ON_ERROR, errs)
        .map(|t| if t == "stop" { OnError::Stop } else { OnError::Node });
    Some(Validation { on_error: on_error? })
}
