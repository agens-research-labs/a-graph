//! Where the content of a prompt or of a media comes from.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_norway::Value;

use super::common::{Errs, Obj};
use crate::error::Rule;
use crate::schema::key;

#[derive(Debug, Clone)]
pub enum Source {
    Path(String),
    Url {
        url: String,
        access: Option<Value>,
    },
    /// The content itself, and its type when the graph gives it.
    Base64 {
        data: String,
        mime_type: Option<String>,
    },
}

/// A source has exactly one of `path`, `url`, `base64`; `access` goes with `url` only, `mime_type` with `base64` only;
/// `base64` must decode.
pub(super) fn build(mut o: Obj, allow_base64: bool, errs: &mut Errs) -> Option<Source> {
    let path = o.opt::<String>(key::PATH, errs);
    let url = o.opt::<String>(key::URL, errs);
    let base64 = o.opt::<String>(key::BASE64, errs);
    let mime_type = o.opt::<String>(key::MIME_TYPE, errs);
    let access = o.value(key::ACCESS);
    let at = o.at().clone();

    let given = [path.is_some(), url.is_some(), base64.is_some()];
    if given.iter().filter(|b| **b).count() != 1 {
        errs.add(&at, Rule::ExactlyOne, "a source needs exactly one of path, url, base64");
        return None;
    }
    errs.forbid(
        &at,
        key::ACCESS,
        access.is_some() && url.is_none(),
        "access goes with url only",
    );
    errs.forbid(
        &at,
        key::MIME_TYPE,
        mime_type.is_some() && base64.is_none(),
        "mime_type goes with base64 only",
    );
    if base64.is_some() && !allow_base64 {
        errs.forbid(&at, key::BASE64, true, "base64 is not allowed here");
        return None;
    }
    if let Some(Err(e)) = base64.as_ref().map(|b| STANDARD.decode(b)) {
        errs.add(
            &at.key(key::BASE64),
            Rule::BadValue,
            format!("`base64` is not valid: {e}"),
        );
    }
    match (path, url, base64) {
        (Some(p), _, _) => Some(Source::Path(p)),
        (_, Some(url), _) => Some(Source::Url { url, access }),
        (_, _, Some(data)) => Some(Source::Base64 { data, mime_type }),
        _ => None,
    }
}
