//! Where the content of a prompt or of a media comes from.

use std::collections::BTreeMap;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use super::common::{Errs, Obj};
use crate::error::{Path, Rule};
use crate::network::Destination;
use crate::schema::key;
use crate::secrets::{self, Secret};

#[derive(Debug, Clone)]
pub enum Source {
    Path(String),
    /// Downloaded when the run starts (`run/sources.rs`).
    Url {
        url: String,
        /// The headers of the request, kept as the name of the environment variable that holds each value.
        headers: BTreeMap<String, String>,
        /// The type given to the utilities; without it, the extension of the path of the url says it.
        mime_type: Option<String>,
    },
    /// The content itself, and its type when the graph gives it.
    Base64 {
        data: String,
        mime_type: Option<String>,
    },
}

/// A source has exactly one of `path`, `url`, `base64`; `headers` goes with `url` only, `mime_type` with `url` or
/// `base64`; `base64` must decode.
pub(super) fn build(mut o: Obj, allow_base64: bool, errs: &mut Errs) -> Option<Source> {
    let path = o.opt::<String>(key::PATH, errs);
    let url = o.opt::<String>(key::URL, errs);
    let base64 = o.opt::<String>(key::BASE64, errs);
    let mime_type = o.opt::<String>(key::MIME_TYPE, errs);
    let has_headers = o.has(key::HEADERS);
    let headers = o.secrets(key::HEADERS, errs);
    let at = o.at().clone();

    let given = [path.is_some(), url.is_some(), base64.is_some()];
    if given.iter().filter(|b| **b).count() != 1 {
        errs.add(&at, Rule::ExactlyOne, "a source needs exactly one of path, url, base64");
        return None;
    }
    errs.forbid(
        &at,
        key::HEADERS,
        has_headers && url.is_none(),
        "headers goes with url only",
    );
    errs.forbid(
        &at,
        key::MIME_TYPE,
        mime_type.is_some() && path.is_some(),
        "mime_type goes with url or base64 only",
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
        (_, Some(url), _) => Some(Source::Url {
            url,
            headers,
            mime_type,
        }),
        (_, _, Some(data)) => Some(Source::Base64 { data, mime_type }),
        _ => None,
    }
}

/// The headers of a source at a url and the url they go to, for `allowed_env`; `at` is the `source`.
pub(super) fn secrets(source: &Source, at: &Path) -> Vec<Secret> {
    match source {
        Source::Url { url, headers, .. } => {
            secrets::headers(headers, at, &Destination::url(at.key(key::URL), url.clone()))
        }
        _ => vec![],
    }
}

/// A header of a source at a url whose environment variable is not set; `at` is the `source`. The values are read
/// again when the url is downloaded.
pub(super) fn missing_env(source: &Source, at: &Path, errs: &mut Errs) {
    if let Source::Url { headers, .. } = source {
        for (name, var) in headers {
            if std::env::var(var).is_err() {
                errs.add(
                    &at.key(key::HEADERS).key(name),
                    Rule::EnvMissing,
                    format!("environment variable `{var}` is not set"),
                );
            }
        }
    }
}
