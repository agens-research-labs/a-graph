//! What every manager shares when it builds: the list of errors and the reading of a list of items.

use std::collections::BTreeMap;

use serde::de::DeserializeOwned;
use serde_norway::{Mapping, Value};

use crate::error::{Error, Errors, Path, Phase, Rule};
use crate::schema::key;

/// The errors found while a manager builds.
#[derive(Default)]
pub(crate) struct Errs(pub(crate) Vec<Error>);

impl Errs {
    pub(crate) fn add(&mut self, path: &Path, rule: Rule, msg: impl Into<String>) {
        self.0.push(Error::new(Phase::Interpretation, path.clone(), rule, msg));
    }

    /// `key` is mandatory when `present` is false.
    pub(crate) fn require(&mut self, at: &Path, key: &str, present: bool, why: &str) {
        if !present {
            self.add(&at.key(key), Rule::MissingField, why);
        }
    }

    /// `key` is not allowed when `present` is true.
    pub(crate) fn forbid(&mut self, at: &Path, key: &str, present: bool, why: &str) {
        if present {
            self.add(&at.key(key), Rule::UnexpectedField, why);
        }
    }

    /// A count (`length`) of at least 1; `value` is what was read, if it could be.
    pub(crate) fn at_least_one(&mut self, at: &Path, value: Option<u32>) {
        if value == Some(0) {
            self.add(at, Rule::BadValue, "at least 1");
        }
    }

    pub(crate) fn finish<T>(self, built: T) -> Result<T, Errors> {
        if self.0.is_empty() { Ok(built) } else { Err(Errors(self.0)) }
    }
}

/// A map of the yml being read by a manager. Every value is taken by the name of its key,
/// always one of the constants of `schema::key`. The Loader has already checked the shape: a
/// mismatch found here is reported, never a panic.
pub(crate) struct Obj {
    map: Mapping,
    at: Path,
}

impl Obj {
    /// The items of a list, each one a map, with its own path.
    pub(crate) fn items(base: &Path, list: Option<Value>, errs: &mut Errs) -> Vec<Obj> {
        let Some(list) = list else { return Vec::new() };
        let Value::Sequence(seq) = list else {
            errs.add(base, Rule::BadValue, "expected a list");
            return Vec::new();
        };
        seq.into_iter().enumerate().filter_map(|(i, v)| Obj::new(v, base.idx(i), errs)).collect()
    }

    pub(crate) fn new(v: Value, at: Path, errs: &mut Errs) -> Option<Obj> {
        match v {
            Value::Mapping(map) => Some(Obj { map, at }),
            _ => {
                errs.add(&at, Rule::BadValue, "expected a map");
                None
            }
        }
    }

    pub(crate) fn at(&self) -> &Path {
        &self.at
    }

    pub(crate) fn has(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    /// An optional value, read as `T`. A value of the wrong type is an error.
    pub(crate) fn opt<T: DeserializeOwned>(&mut self, key: &str, errs: &mut Errs) -> Option<T> {
        let v = self.map.remove(key)?;
        // serde_norway reads an enum as a YAML tag; the yml writes it as a one-key map.
        match serde_norway::with::singleton_map_recursive::deserialize::<T, _>(v) {
            Ok(t) => Some(t),
            Err(e) => {
                errs.add(&self.at.key(key), Rule::BadValue, e.to_string());
                None
            }
        }
    }

    /// A mandatory value: absent is an error.
    pub(crate) fn req<T: DeserializeOwned>(&mut self, key: &str, errs: &mut Errs) -> Option<T> {
        if !self.has(key) {
            errs.add(&self.at.key(key), Rule::MissingField, format!("`{key}` is mandatory"));
            return None;
        }
        self.opt(key, errs)
    }

    /// A text; a number is accepted as its text (`512`).
    pub(crate) fn opt_text(&mut self, key: &str, errs: &mut Errs) -> Option<String> {
        match self.map.remove(key)? {
            Value::String(s) => Some(s),
            Value::Number(n) => Some(n.to_string()),
            _ => {
                errs.add(&self.at.key(key), Rule::BadValue, "expected a text");
                None
            }
        }
    }

    /// A map of exactly one key (`- next: B`, `- parallel: [...]`): the key and its value.
    pub(crate) fn single(self, errs: &mut Errs) -> Option<(String, Value)> {
        if self.map.len() != 1 {
            errs.add(&self.at, Rule::ExactlyOne, "an entry has exactly one key");
            return None;
        }
        let (k, v) = self.map.into_iter().next()?;
        match k {
            Value::String(k) => Some((k, v)),
            _ => {
                errs.add(&self.at, Rule::BadValue, "a key must be a text");
                None
            }
        }
    }

    /// The value as it is written, whatever it is.
    pub(crate) fn value(&mut self, key: &str) -> Option<Value> {
        self.map.remove(key)
    }

    /// A nested map, optional.
    pub(crate) fn sub(&mut self, key: &str, errs: &mut Errs) -> Option<Obj> {
        let v = self.map.remove(key)?;
        Obj::new(v, self.at.key(key), errs)
    }

    /// A nested map, mandatory.
    pub(crate) fn req_sub(&mut self, key: &str, errs: &mut Errs) -> Option<Obj> {
        if !self.has(key) {
            errs.add(&self.at.key(key), Rule::MissingField, format!("`{key}` is mandatory"));
            return None;
        }
        self.sub(key, errs)
    }

    /// `{ env: VAR_NAME }`: returns the NAME of the variable, never a value.
    pub(crate) fn opt_secret(&mut self, key: &str, errs: &mut Errs) -> Option<String> {
        self.sub(key, errs)?.req::<String>(key::ENV, errs)
    }

    pub(crate) fn req_secret(&mut self, key: &str, errs: &mut Errs) -> Option<String> {
        self.req_sub(key, errs)?.req::<String>(key::ENV, errs)
    }

    /// A map of any names to `{ env: VAR_NAME }` (headers, environment of a process).
    pub(crate) fn secrets(&mut self, key: &str, errs: &mut Errs) -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        let Some(map) = self.sub(key, errs) else { return out };
        for (name, v) in map.map {
            let Some(name) = name.as_str().map(str::to_string) else { continue };
            if let Some(mut s) = Obj::new(v, map.at.key(&name), errs)
                && let Some(var) = s.req::<String>(key::ENV, errs)
            {
                out.insert(name, var);
            }
        }
        out
    }
}
