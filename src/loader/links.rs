//! The links between ids: an id is defined once and every citation of it must find a definition.

use std::collections::{HashMap, HashSet};

use super::check::Collector;
use crate::error::{Error, Phase, Rule};
use crate::schema::Kind;

pub(super) fn resolve(c: &mut Collector) {
    // a name is unique in the whole yml, whatever it names
    let mut used: HashMap<&str, Kind> = HashMap::new();
    let mut defined: HashMap<Kind, HashSet<&str>> = HashMap::new();
    let mut errs = Vec::new();

    for (kind, id, path) in &c.defs {
        match used.get(id.as_str()) {
            Some(first) => errs.push(Error::new(
                Phase::Validation,
                path.clone(),
                Rule::DuplicateId,
                format!("`{id}` is already used (as a {})", first.name()),
            )),
            None => {
                used.insert(id, *kind);
                defined.entry(*kind).or_default().insert(id);
            }
        }
    }
    for (kind, id, path) in &c.refs {
        if !defined.get(kind).is_some_and(|ids| ids.contains(id.as_str())) {
            errs.push(Error::new(
                Phase::Validation,
                path.clone(),
                Rule::UnknownRef,
                format!("{} `{id}` is not declared", kind.name()),
            ));
        }
    }
    c.errs.extend(errs);
}
