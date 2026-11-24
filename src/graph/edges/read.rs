//! Reading `graph.edges` with the names of `schema::key`, and the rules about its content.

use serde_norway::Value;

use super::{Edges, Group, Pattern, React, Step};
use crate::build::{Errs, Obj};
use crate::error::{Errors, Path, Rule};
use crate::graph::nodes::{Entity, Nodes};
use crate::schema::key;

struct Reader<'a> {
    nodes: &'a Nodes,
    errs: Errs,
}

pub(super) fn build(list: Option<Value>, nodes: &Nodes) -> Result<Edges, Errors> {
    let mut r = Reader {
        nodes,
        errs: Errs::default(),
    };
    let base = Path::root().key(key::GRAPH).key(key::EDGES);
    let entries = Obj::items(&base, list, &mut r.errs);
    let entries_len = entries.len();

    let (mut start, mut end, mut steps) = (None, None, Vec::new());
    // whether the first and the last entry are a group or a `react`, which may stand without `start` or `end`
    let (mut first_block, mut last_block) = (false, false);
    for (i, o) in entries.into_iter().enumerate() {
        let at = o.at().clone();
        let Some((k, v)) = o.single(&mut r.errs) else { continue };
        let at_key = at.key(&k);
        match k.as_str() {
            key::START => start = r.node(&v, &at_key),
            key::END => end = r.node(&v, &at_key),
            _ => {
                let step = r.step(&k, v, &at_key);
                let block = matches!(step, Some(Step::Group(_) | Step::React(_)));
                first_block |= i == 0 && block;
                last_block = i + 1 == entries_len && block;
                steps.extend(step);
            }
        }
    }
    // the Loader has already checked the rules; a yml that skipped it is refused here
    // a graph of one node is the `start` alone, with no `end` and nothing else
    let alone = start.is_some() && end.is_none() && steps.is_empty() && entries_len == 1;
    let complete = (start.is_some() || first_block) && (end.is_some() || alone || last_block);
    if !complete || !r.errs.0.is_empty() {
        if r.errs.0.is_empty() {
            r.errs.add(
                &base,
                Rule::BlockShape,
                "the edges have no `start` or no `end` where they need one: they were not validated by the Loader",
            );
        }
        return Err(Errors(r.errs.0));
    }

    Ok(Edges { start, end, steps })
}

impl Reader<'_> {
    /// The entity of the node id written in `v`.
    fn node(&mut self, v: &Value, at: &Path) -> Option<Entity> {
        let Some(id) = v.as_str() else {
            self.errs.add(at, Rule::BadValue, "expected the id of a node");
            return None;
        };
        let e = self.nodes.entity(id);
        if e.is_none() {
            self.errs
                .add(at, Rule::UnknownRef, format!("node `{id}` is not declared"));
        }
        e
    }

    /// A step: `next`, a group or a `react`. In a `react`, a plain node is a `worker`.
    fn step(&mut self, k: &str, v: Value, at: &Path) -> Option<Step> {
        self.step_of(k, v, at, key::NEXT)
    }

    fn step_of(&mut self, k: &str, v: Value, at: &Path, plain: &str) -> Option<Step> {
        match k {
            _ if k == plain => self.node(&v, at).map(Step::Node),
            key::LINEAR => self.group(Pattern::Linear, v, at).map(Step::Group),
            key::PARALLEL => self.group(Pattern::Parallel, v, at).map(Step::Group),
            key::CONCURRENT => self.group(Pattern::Concurrent, v, at).map(Step::Group),
            key::BATCH => self.group(Pattern::Batch, v, at).map(Step::Group),
            key::REACT => self.react(v, at).map(Step::React),
            other => {
                self.errs
                    .add(at, Rule::UnexpectedField, format!("`{other}` is not allowed here"));
                None
            }
        }
    }

    /// A list of entries (the `fan` of a group, the `components` of a `react`).
    fn steps(&mut self, list: Value, at: &Path, plain: &str) -> Vec<Step> {
        let mut out = Vec::new();
        for o in Obj::items(at, Some(list), &mut self.errs) {
            let entry = o.at().clone();
            let Some((k, v)) = o.single(&mut self.errs) else {
                continue;
            };
            if let Some(step) = self.step_of(&k, v, &entry.key(&k), plain) {
                out.push(step);
            }
        }
        out
    }

    /// `[limit], in, fan, [out]`. The rules about them are the Loader's.
    fn group(&mut self, pattern: Pattern, v: Value, at: &Path) -> Option<Group> {
        let (mut limit, mut r#in, mut fan, mut out) = (None, None, Vec::new(), None);
        for o in Obj::items(at, Some(v), &mut self.errs) {
            let entry = o.at().clone();
            let Some((k, v)) = o.single(&mut self.errs) else {
                continue;
            };
            let here = entry.key(&k);
            match k.as_str() {
                key::LIMIT => limit = serde_norway::from_value::<u32>(v).ok(),
                key::IN => r#in = self.node(&v, &here),
                key::FAN => fan = self.steps(v, &here, key::NEXT),
                key::OUT => out = self.node(&v, &here),
                _ => {}
            }
        }
        Some(Group {
            pattern,
            limit,
            r#in: r#in?,
            fan,
            out,
        })
    }

    fn react(&mut self, v: Value, at: &Path) -> Option<React> {
        let mut o = Obj::new(v, at.clone(), &mut self.errs)?;
        let controller = o.req::<String>(key::CONTROLLER, &mut self.errs);
        let max_iterations = o.req::<u32>(key::MAX_ITERATIONS, &mut self.errs);
        let structured_output = crate::graph::nodes::structured_output(&mut o, &mut self.errs);
        let components = o.value(key::COMPONENTS);
        let controller = controller.and_then(|id| self.node(&Value::String(id), &at.key(key::CONTROLLER)));
        let workers = components
            .map(|c| self.steps(c, &at.key(key::COMPONENTS), key::WORKER))
            .unwrap_or_default();
        Some(React {
            controller: controller?,
            max_iterations: max_iterations?,
            structured_output: structured_output?,
            workers,
        })
    }
}
