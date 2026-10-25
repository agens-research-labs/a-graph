//! Walks the yml together with the `schema` shape of every value.

use serde_norway::{Mapping, Value};

use crate::error::{Error, Path, Phase, Rule};
use crate::schema::{Field, Kind, Shape};

/// What the walk finds: the errors, and the ids defined and cited (resolved by `links`).
#[derive(Default)]
pub(super) struct Collector {
    pub(super) errs: Vec<Error>,
    pub(super) defs: Vec<(Kind, String, Path)>,
    pub(super) refs: Vec<(Kind, String, Path)>,
}

impl Collector {
    fn err(&mut self, path: &Path, rule: Rule, msg: impl Into<String>) {
        self.errs.push(Error::new(Phase::Validation, path.clone(), rule, msg));
    }
}

fn kind_of(v: &Value) -> &'static str {
    match v {
        Value::Null => "nothing",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a text",
        Value::Sequence(_) => "a list",
        Value::Mapping(_) => "a map",
        Value::Tagged(_) => "a tagged value",
    }
}

/// The text of a scalar: a word can be written as a number (`512`).
fn word(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

pub(super) fn check(v: &Value, shape: &Shape, path: &Path, c: &mut Collector) {
    match shape {
        Shape::Any => {}
        Shape::Lazy { get, .. } => check(v, get(), path, c),
        Shape::Text => {
            if !v.is_string() {
                c.err(path, Rule::BadValue, format!("expected a text, found {}", kind_of(v)));
            }
        }
        Shape::Int => {
            if !matches!(v, Value::Number(n) if n.is_i64() || n.is_u64()) {
                c.err(path, Rule::BadValue, format!("expected a whole number, found {}", kind_of(v)));
            }
        }
        Shape::Number => {
            if !v.is_number() {
                c.err(path, Rule::BadValue, format!("expected a number, found {}", kind_of(v)));
            }
        }
        Shape::Bool => {
            if !v.is_bool() {
                c.err(path, Rule::BadValue, format!("expected true or false, found {}", kind_of(v)));
            }
        }
        Shape::Words(words) => match word(v) {
            Some(w) if words.contains(&w.as_str()) => {}
            _ => c.err(path, Rule::BadValue, format!("expected one of: {}", words.join(", "))),
        },
        Shape::Id(kind) => match v.as_str() {
            Some(id) => c.defs.push((*kind, id.to_string(), path.clone())),
            None => c.err(path, Rule::BadValue, format!("expected the id of a {}, found {}", kind.name(), kind_of(v))),
        },
        Shape::Ref(kind) => match v.as_str() {
            Some(id) => c.refs.push((*kind, id.to_string(), path.clone())),
            None => c.err(path, Rule::BadValue, format!("expected the id of a {}, found {}", kind.name(), kind_of(v))),
        },
        Shape::List(inner) => match v.as_sequence() {
            Some(items) => {
                for (i, item) in items.iter().enumerate() {
                    check(item, inner, &path.idx(i), c);
                }
            }
            None => c.err(path, Rule::BadValue, format!("expected a list, found {}", kind_of(v))),
        },
        Shape::Map(fields) => match v.as_mapping() {
            Some(m) => check_map(m, &[fields], None, path, c),
            None => c.err(path, Rule::BadValue, format!("expected a map, found {}", kind_of(v))),
        },
        Shape::MapOf(inner) => match v.as_mapping() {
            Some(m) => {
                for (k, val) in m {
                    match k.as_str() {
                        Some(k) => check(val, inner, &path.key(k), c),
                        None => c.err(path, Rule::BadValue, "a key must be a text"),
                    }
                }
            }
            None => c.err(path, Rule::BadValue, format!("expected a map, found {}", kind_of(v))),
        },
        Shape::Tagged { by, common, variants, .. } => {
            let Some(m) = v.as_mapping() else {
                c.err(path, Rule::BadValue, format!("expected a map, found {}", kind_of(v)));
                return;
            };
            let names = variants.iter().map(|variant| variant.name).collect::<Vec<_>>().join(", ");
            match m.get(*by).and_then(word) {
                None => c.err(&path.key(by), Rule::MissingField, format!("`{by}` is missing; one of: {names}")),
                Some(tag) => match variants.iter().find(|variant| variant.name == tag) {
                    None => c.err(&path.key(by), Rule::BadValue, format!("`{tag}` is not a {by}; one of: {names}")),
                    Some(variant) => check_map(m, &[common, variant.fields], Some(by), path, c),
                },
            }
        }
        Shape::Either(alternatives) => {
            // The first alternative without errors wins; if none, report the one closest to fit.
            let mut best: Option<(usize, usize)> = None;
            for (i, alt) in alternatives.iter().enumerate() {
                let mut scratch = Collector::default();
                check(v, alt, path, &mut scratch);
                if scratch.errs.is_empty() {
                    best = Some((i, 0));
                    break;
                }
                if best.map_or(true, |(_, n)| scratch.errs.len() < n) {
                    best = Some((i, scratch.errs.len()));
                }
            }
            if let Some((i, _)) = best {
                check(v, &alternatives[i], path, c);
            }
        }
    }
}

/// A map checked against one or more lists of fields; `tag` is a key already validated.
fn check_map(m: &Mapping, field_lists: &[&[Field]], tag: Option<&str>, path: &Path, c: &mut Collector) {
    let find = |name: &str| field_lists.iter().flat_map(|l| l.iter()).find(|f| f.name == name);
    for (k, val) in m {
        let Some(k) = k.as_str() else {
            c.err(path, Rule::BadValue, "a key must be a text");
            continue;
        };
        if Some(k) == tag {
            continue;
        }
        match find(k) {
            Some(f) => check(val, &f.shape, &path.key(k), c),
            None => c.err(&path.key(k), Rule::UnexpectedField, format!("`{k}` is not a key allowed here")),
        }
    }
    for f in field_lists.iter().flat_map(|l| l.iter()).filter(|f| f.required) {
        if !m.contains_key(f.name) {
            c.err(&path.key(f.name), Rule::MissingField, format!("`{}` is mandatory", f.name));
        }
    }
}
