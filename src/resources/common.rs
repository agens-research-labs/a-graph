//! What the resource managers share.

use serde::Deserialize;

pub(super) use crate::build::{Errs, Obj};
use crate::error::Path;
use crate::schema::key;

/// Where the list `resources.<list>` is in the yml.
pub(super) fn list_path(list: &str) -> Path {
    Path::root().key(key::RESOURCES).key(list)
}

/// The value of `transport`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum RawTransport {
    Stdio,
    Http,
}
