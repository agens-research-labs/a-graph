//! Which parts of an event may be written, decided by the passport.

use serde::Deserialize;
use serde_norway::Value;

use crate::build::{Errs, Obj};
use crate::error::{Errors, Path};
use crate::schema::key;

/// The value of `data_classification.sensitivity`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Sensitivity {
    Public,
    Internal,
    Confidential,
    Restricted,
}

/// Message contents appear only if the data is `public` and no category is declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Redaction {
    pub allow_content: bool,
}

impl Redaction {
    /// Reads the `passport` block: `adl.data_classification`.
    pub fn from_passport(passport: Option<Value>) -> Result<Self, Errors> {
        let mut errs = Errs::default();
        let read = |passport: Value, errs: &mut Errs| -> Option<bool> {
            let mut p = Obj::new(passport, Path::root().key(key::PASSPORT), errs)?;
            let mut dc = p.req_sub(key::ADL, errs)?.req_sub(key::DATA_CLASSIFICATION, errs)?;
            let sensitivity = dc.req::<Sensitivity>(key::SENSITIVITY, errs)?;
            let categories = dc.opt::<Vec<String>>(key::CATEGORIES, errs).unwrap_or_default();
            Some(sensitivity == Sensitivity::Public && categories.is_empty())
        };
        let allow_content = match passport {
            Some(p) => read(p, &mut errs),
            None => {
                errs.add(&Path::root().key(key::PASSPORT), crate::error::Rule::MissingField, "`passport` is mandatory");
                None
            }
        };
        errs.finish(Redaction { allow_content: allow_content.unwrap_or(false) })
    }
}
