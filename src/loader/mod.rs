//! The Loader: reads the yml and validates its coherence. It checks that the yml is well formed,
//! that the nomenclature of the `schema` is respected and that the links between ids are correct.
//! The checks on the content of each block are made by its manager, when it builds.

mod check;
mod edges;
mod links;

use serde_norway::{Mapping, Value};

use crate::error::{Error, Errors, Path, Phase, Rule};
use crate::schema::{self, Field, Shape};
use check::Collector;

/// A yml that passed the Loader. Each block is handed over to its manager, moved, not copied.
#[derive(Debug)]
pub struct Document {
    blocks: Mapping,
}

impl Document {
    /// Hands over a block (`schema::key::MODELS`, ...). `None` if absent or already taken.
    pub fn take(&mut self, block: &str) -> Option<Value> {
        self.blocks.remove(block)
    }
}

/// Parses `src` and validates it. On failure returns every error found, not only the first.
pub fn load(src: &str) -> Result<Document, Errors> {
    let root: Value = serde_norway::from_str(src).map_err(|e| {
        Errors(vec![Error::new(
            Phase::Validation,
            Path::root(),
            Rule::Parse,
            e.to_string(),
        )])
    })?;
    load_value(root)
}

/// Validates a yml already parsed, as `load` does once it has parsed it: the service parses the yml itself, to
/// check its rules, and hands the tree over instead of writing it again as text.
pub fn load_value(root: Value) -> Result<Document, Errors> {
    let mut c = Collector::default();
    static DOCUMENT: Shape = Shape::Map(schema::DOCUMENT);
    check::check(&root, &DOCUMENT, &Path::root(), &mut c);
    links::resolve(&mut c);
    edges::check(&root, &mut c);

    match (c.errs.is_empty(), root) {
        (true, Value::Mapping(blocks)) => Ok(Document { blocks }),
        _ => Err(Errors(c.errs)),
    }
}

/// Checks a file that is not a graph (`schema::SERVICE`) against its keys and the kinds of their
/// values. Every error found; there are no ids to link.
pub fn check_shape(root: &Value, fields: &'static [Field]) -> Vec<Error> {
    let mut c = Collector::default();
    check::check(root, &Shape::Map(fields), &Path::root(), &mut c);
    c.errs
}
