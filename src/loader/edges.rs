//! The coherence of `graph.edges`, which the shapes of the schema cannot say: the entries and their
//! order, how a group is made (`limit` is mandatory in `concurrent`), when a group may have no `out`, and where a validation node may stand and pass its problem on. Names and the nodes cited are
//! already checked by the shapes; here the yml is read as it is written, and what does not have
//! the expected shape is skipped (the shapes have already reported it).

use serde_norway::{Mapping, Value};

use super::check::Collector;
use crate::error::{Error, Path, Phase, Rule};
use crate::schema::key;

const GROUPS: [&str; 4] = [key::LINEAR, key::PARALLEL, key::CONCURRENT, key::BATCH];
const GROUP_ORDER: [&str; 4] = [key::LIMIT, key::IN, key::FAN, key::OUT];

pub(super) fn check(root: &Value, c: &mut Collector) {
    let Some(graph) = root.get(key::GRAPH) else { return };
    let Some(Value::Sequence(entries)) = graph.get(key::EDGES) else { return };
    let nodes = Nodes::new(graph.get(key::NODES));
    let base = Path::root().key(key::GRAPH).key(key::EDGES);

    let (mut starts, mut ends) = (0, 0);
    let mut start_id: Option<&str> = None;
    for (i, entry) in entries.iter().enumerate() {
        let Some((k, v)) = single(entry) else { continue };
        let at = base.idx(i).key(k);
        match k {
            key::START => {
                starts += 1;
                start_id = v.as_str();
                if starts > 1 {
                    c.errs.push(error(&at, Rule::EdgeStart, "the graph needs exactly one `start`"));
                }
                if i != 0 {
                    c.errs.push(error(&at, Rule::EdgeStart, "`start` must be the first entry"));
                }
            }
            key::END => {
                ends += 1;
                if let Some(id) = v.as_str() {
                    nodes.nothing_after(id, "it is the `end` node", c);
                }
                if ends > 1 {
                    c.errs.push(error(&at, Rule::EdgeEnd, "the graph needs exactly one `end`"));
                }
                if i != entries.len() - 1 {
                    c.errs.push(error(&at, Rule::EdgeEnd, "`end` must be the last entry"));
                }
            }
            _ => {
                step(k, v, &at, &nodes, c);
                // a judge is a step of the main flow, never inside a group or a react
                if GROUPS.contains(&k) || k == key::REACT {
                    let mut members = Vec::new();
                    members_of(&Value::Sequence(vec![entry.clone()]), &mut members);
                    for id in members {
                        nodes.judge_in_group(&id, c);
                    }
                }
            }
        }
    }
    if starts == 0 {
        c.errs.push(error(&base, Rule::EdgeStart, "the graph needs exactly one `start`"));
    }
    if ends == 0 {
        // a graph of one node is the `start` alone: the node runs once and its output is the result
        if starts == 1 && entries.len() == 1 {
            if let Some(id) = start_id {
                nodes.nothing_after(id, "it is the only node of the graph", c);
            }
        } else {
            c.errs.push(error(&base, Rule::EdgeEnd, "the graph needs exactly one `end` (only a graph of one node, a `start` alone, has none)"));
        }
    }
}

fn error(at: &Path, rule: Rule, msg: impl Into<String>) -> Error {
    Error::new(Phase::Validation, at.clone(), rule, msg)
}

/// A map of exactly one key: the key and its value.
fn single(v: &Value) -> Option<(&str, &Value)> {
    let m = v.as_mapping()?;
    if m.len() != 1 {
        return None;
    }
    let (k, v) = m.iter().next()?;
    Some((k.as_str()?, v))
}

/// An entry of a list: a group, a `react`, or a plain node.
fn step(k: &str, v: &Value, at: &Path, nodes: &Nodes, c: &mut Collector) {
    if GROUPS.contains(&k) {
        group(k, v, at, nodes, c);
    } else if k == key::REACT {
        react(v, at, nodes, c);
    }
}

fn steps(list: &Value, at: &Path, nodes: &Nodes, c: &mut Collector) {
    let Some(items) = list.as_sequence() else { return };
    for (i, item) in items.iter().enumerate() {
        if let Some((k, v)) = single(item) {
            step(k, v, &at.idx(i).key(k), nodes, c);
        }
    }
}

/// The groups inside the `components`; the controller has no tools, MCP servers, skills, blackboard
/// or output schema of its own; the workers have different ids.
fn react(v: &Value, at: &Path, nodes: &Nodes, c: &mut Collector) {
    if let Some(controller) = v.get(key::CONTROLLER).and_then(Value::as_str) {
        nodes.controller_rules(controller, c);
    }
    if let Some(components) = v.get(key::COMPONENTS) {
        let here = at.key(key::COMPONENTS);
        steps(components, &here, nodes, c);
        let mut seen: Vec<String> = Vec::new();
        for (i, item) in components.as_sequence().into_iter().flatten().enumerate() {
            let Some(id) = worker_id(item) else { continue };
            if seen.contains(&id) {
                c.errs.push(error(&here.idx(i), Rule::DuplicateId, format!("two workers are called `{id}`: the controller could not tell which to call")));
            }
            seen.push(id);
        }
    }
}

/// The name the controller calls a worker by: the node, or the `in` node of a group.
fn worker_id(item: &Value) -> Option<String> {
    let (k, v) = single(item)?;
    if k == key::WORKER {
        return v.as_str().map(str::to_string);
    }
    if GROUPS.contains(&k) {
        return v.as_sequence()?.iter().find_map(|e| match single(e) {
            Some((key::IN, id)) => id.as_str().map(str::to_string),
            _ => None,
        });
    }
    None
}

/// `[limit], in, fan, [out]`, each once and in this order; `fan` not empty; no `limit` in `linear`.
fn group(kind: &str, v: &Value, at: &Path, nodes: &Nodes, c: &mut Collector) {
    let Some(items) = v.as_sequence() else { return };
    let mut order: Vec<&str> = Vec::new();
    let mut fan: Option<&Value> = None;
    for item in items {
        let Some((k, value)) = single(item) else { return };
        order.push(k);
        if k == key::FAN {
            fan = Some(value);
        }
    }
    for field in GROUP_ORDER {
        if order.iter().filter(|f| **f == field).count() > 1 {
            c.errs.push(error(at, Rule::ExactlyOne, format!("`{field}` is written once in a group")));
        }
    }
    for field in [key::IN, key::FAN] {
        if !order.contains(&field) {
            c.errs.push(error(&at.key(field), Rule::MissingField, format!("`{field}` is mandatory in a group")));
        }
    }
    let rank = |f: &str| GROUP_ORDER.iter().position(|x| *x == f);
    if order.windows(2).any(|w| rank(w[0]) > rank(w[1])) {
        c.errs.push(error(at, Rule::BlockShape, "the entries of a group go in this order: limit, in, fan, out"));
    }
    if kind == key::LINEAR && order.contains(&key::LIMIT) {
        c.errs.push(error(&at.key(key::LIMIT), Rule::UnexpectedField, "`limit` is for parallel, concurrent and batch"));
    }
    if kind == key::CONCURRENT && !order.contains(&key::LIMIT) {
        c.errs.push(error(&at.key(key::LIMIT), Rule::MissingField, "`limit` is mandatory in a concurrent group: it is how many branches run at the same time"));
    }
    let Some(fan) = fan else { return };
    if fan.as_sequence().is_some_and(|s| s.is_empty()) {
        c.errs.push(error(&at.key(key::FAN), Rule::BadValue, "`fan` needs at least one branch"));
    }

    // the branches may be groups: they are checked too
    let fan_at = fan_path(at, items);
    steps(fan, &fan_at, nodes, c);

    // a group may omit `out` only if none of its nodes passes its output on
    if !order.contains(&key::OUT) {
        let mut members = Vec::new();
        members_of(fan, &mut members);
        for id in members {
            nodes.forwarding(&id, c);
        }
    }
}

/// The path of the `fan` entry: the position in the list of the group.
fn fan_path(at: &Path, items: &[Value]) -> Path {
    let i = items.iter().position(|it| single(it).is_some_and(|(k, _)| k == key::FAN)).unwrap_or(0);
    at.idx(i).key(key::FAN)
}

/// The ids of the nodes in a `fan`, the nested groups and the `react` included.
fn members_of(list: &Value, out: &mut Vec<String>) {
    let Some(items) = list.as_sequence() else { return };
    for item in items {
        let Some((k, v)) = single(item) else { continue };
        if k == key::NEXT || k == key::WORKER {
            out.extend(v.as_str().map(str::to_string));
        } else if GROUPS.contains(&k) {
            for entry in v.as_sequence().into_iter().flatten() {
                match single(entry) {
                    Some((key::IN | key::OUT, id)) => out.extend(id.as_str().map(str::to_string)),
                    Some((key::FAN, fan)) => members_of(fan, out),
                    _ => {}
                }
            }
        } else if k == key::REACT {
            out.extend(v.get(key::CONTROLLER).and_then(Value::as_str).map(str::to_string));
            if let Some(components) = v.get(key::COMPONENTS) {
                members_of(components, out);
            }
        }
    }
}

/// The nodes as written: whether one passes its output on.
struct Nodes<'a> {
    list: Vec<(&'a str, &'a Mapping)>,
}

impl<'a> Nodes<'a> {
    fn new(nodes: Option<&'a Value>) -> Self {
        let list = nodes
            .and_then(Value::as_sequence)
            .into_iter()
            .flatten()
            .filter_map(|n| {
                let m = n.as_mapping()?;
                Some((m.get(key::ID)?.as_str()?, m))
            })
            .collect();
        Nodes { list }
    }

    /// A controller answers with the routing output that the react defines, and does not mix it
    /// with tools, MCP servers, skills or a blackboard.
    fn controller_rules(&self, id: &str, c: &mut Collector) {
        let Some((index, (_, node))) = self.list.iter().enumerate().find(|(_, (i, _))| *i == id) else { return };
        let at = Path::root().key(key::GRAPH).key(key::NODES).idx(index);
        let resources = node.get(key::RESOURCES);
        for field in [key::TOOLS, key::MCP_SERVERS, key::SKILLS] {
            let used = resources.and_then(|r| r.get(field)).and_then(Value::as_sequence).is_some_and(|l| !l.is_empty());
            if used {
                c.errs.push(error(&at.key(key::RESOURCES).key(field), Rule::UnexpectedField, format!("node `{id}` is a react controller: it cannot have tools, MCP servers or skills; give them to a worker")));
            }
        }
        if node.get(key::BLACKBOARD).is_some() {
            c.errs.push(error(&at.key(key::BLACKBOARD), Rule::UnexpectedField, format!("node `{id}` is a react controller: it cannot have a blackboard; give it to a worker")));
        }
        if node.get(key::MESSAGE).and_then(|m| m.get(key::OUTPUT)).and_then(|o| o.get(key::SCHEMA)).is_some() {
            c.errs.push(error(&at.key(key::MESSAGE).key(key::OUTPUT).key(key::SCHEMA), Rule::UnexpectedField, format!("node `{id}` is a react controller: its output is defined by the react")));
        }
    }

    /// A validation node that passes the problem to the next node (`on_error: node`) needs a next
    /// node: with none, the problem would go to the final output, which `stop` says honestly.
    fn nothing_after(&self, id: &str, why: &str, c: &mut Collector) {
        let Some((index, (_, node))) = self.list.iter().enumerate().find(|(_, (i, _))| *i == id) else { return };
        let passes = node.get(key::VALIDATION_NODE).and_then(|v| v.get(key::ON_ERROR)).and_then(Value::as_str) == Some("node");
        if passes {
            let at = Path::root().key(key::GRAPH).key(key::NODES).idx(index).key(key::VALIDATION_NODE).key(key::ON_ERROR);
            c.errs.push(error(&at, Rule::ValidationNoNext, format!("node `{id}` has `on_error: node` but {why}, so no node follows it: use `stop`")));
        }
    }

    /// A validation node is a step of the main flow: in a group (as `in`, `out` or in the `fan`) or
    /// in a `react` its place in the run would not be a single point.
    fn judge_in_group(&self, id: &str, c: &mut Collector) {
        let Some((index, (_, node))) = self.list.iter().enumerate().find(|(_, (i, _))| *i == id) else { return };
        if node.get(key::VALIDATION_NODE).is_some() {
            let at = Path::root().key(key::GRAPH).key(key::NODES).idx(index).key(key::VALIDATION_NODE);
            c.errs.push(error(&at, Rule::ValidationInGroup, format!("node `{id}` is a validation node: it goes in the main flow (`start`, `next`, `end`), not in a group or a react")));
        }
    }

    /// An error for `message.output.node` and for each media of `media.output` with `node` true.
    fn forwarding(&self, id: &str, c: &mut Collector) {
        let Some((index, (_, node))) = self.list.iter().enumerate().find(|(_, (i, _))| *i == id) else { return };
        let base = Path::root().key(key::GRAPH).key(key::NODES).idx(index);
        let refuse = |at: Path, c: &mut Collector| c.errs.push(error(&at, Rule::GroupOutRequired, format!("node `{id}` passes its output on, but its group has no `out`")));
        let message_node = node.get(key::MESSAGE).and_then(|b| b.get(key::OUTPUT)).and_then(|o| o.get(key::NODE)).and_then(Value::as_bool) == Some(true);
        if message_node {
            refuse(base.key(key::MESSAGE).key(key::OUTPUT).key(key::NODE), c);
        }
        if let Some(entries) = node.get(key::MEDIA).and_then(|b| b.get(key::OUTPUT)).and_then(Value::as_sequence) {
            for (i, entry) in entries.iter().enumerate() {
                if entry.get(key::NODE).and_then(Value::as_bool) == Some(true) {
                    refuse(base.key(key::MEDIA).key(key::OUTPUT).idx(i).key(key::NODE), c);
                }
            }
        }
    }
}
