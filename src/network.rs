//! The network permission: every destination the file reaches must be in
//! `passport.adl.permissions.network.allowed_hosts`. Absent `permissions` = nothing is allowed.
//! stdout, files and the verbose display are not network. Hosts, not full URLs.
//!
//! The managers say where they go (`destinations()`); this module only compares.

use std::collections::HashSet;

use serde_norway::Value;

use crate::build::{Errs, Obj};
use crate::error::{Error, Errors, Path, Phase, Rule};
use crate::net::host_of;
use crate::schema::key;

/// Where a network call goes.
#[derive(Debug, Clone)]
pub enum Target {
    /// A URL written in the yml.
    Url(String),
    /// The host a provider is reached at; `Err` says why it cannot be known.
    Host(Result<String, String>),
}

/// A destination and where it is written in the yml.
#[derive(Debug, Clone)]
pub struct Destination {
    pub path: Path,
    pub target: Target,
}

impl Destination {
    pub fn url(path: Path, url: impl Into<String>) -> Self {
        Destination { path, target: Target::Url(url.into()) }
    }

    pub fn host(path: Path, host: Result<String, String>) -> Self {
        Destination { path, target: Target::Host(host) }
    }
}

/// `passport.adl.permissions.network.allowed_hosts`, lowercase.
pub fn allowed_hosts(passport: Option<Value>) -> Result<HashSet<String>, Errors> {
    let mut errs = Errs::default();
    let read = |passport: Value, errs: &mut Errs| -> Option<HashSet<String>> {
        let mut p = Obj::new(passport, Path::root().key(key::PASSPORT), errs)?;
        let mut adl = p.req_sub(key::ADL, errs)?;
        let hosts = adl.sub(key::PERMISSIONS, errs).and_then(|mut perm| perm.sub(key::NETWORK, errs)).and_then(|mut n| n.opt::<Vec<String>>(key::ALLOWED_HOSTS, errs));
        Some(hosts.unwrap_or_default().into_iter().map(|h| h.to_lowercase()).collect())
    };
    let hosts = passport.and_then(|p| read(p, &mut errs));
    errs.finish(hosts.unwrap_or_default())
}

/// Every destination whose host is not allowed, or cannot be read, is an error.
pub fn check(allowed: &HashSet<String>, destinations: impl IntoIterator<Item = Destination>) -> Vec<Error> {
    let not_allowed = "is not in passport.adl.permissions.network.allowed_hosts";
    let mut errors = Vec::new();
    for d in destinations {
        let host = match &d.target {
            Target::Url(url) => host_of(url).ok_or_else(|| format!("`{url}` is not a URL with a host")),
            Target::Host(h) => h.clone(),
        };
        match host {
            Err(why) => errors.push(Error::new(Phase::Interpretation, d.path, Rule::BadValue, why)),
            Ok(h) if !allowed.contains(&h) => errors.push(Error::new(Phase::Interpretation, d.path, Rule::NetworkNotAllowed, format!("host `{h}` {not_allowed}"))),
            Ok(_) => {}
        }
    }
    errors
}

/// The destinations of the logging: `observability.output.url` and `trace.endpoint`.
pub fn logging_destinations(observability: Option<Value>, trace: Option<Value>) -> Result<Vec<Destination>, Errors> {
    let mut errs = Errs::default();
    let mut out = Vec::new();
    let obs_path = Path::root().key(key::OBSERVABILITY).key(key::OUTPUT);
    if let Some(Value::Mapping(mut obs)) = observability {
        // `stdout` is a word and `{ file }` has no url: only `{ url }` is network
        if let Some(Value::Mapping(output)) = obs.remove(key::OUTPUT) {
            if let Some(mut o) = Obj::new(Value::Mapping(output), obs_path.clone(), &mut errs) {
                if let Some(url) = o.opt::<String>(key::URL, &mut errs) {
                    out.push(Destination::url(obs_path.key(key::URL), url));
                }
            }
        }
    }
    if let Some(t) = trace {
        if let Some(mut t) = Obj::new(t, Path::root().key(key::TRACE), &mut errs) {
            if let Some(endpoint) = t.opt::<String>(key::ENDPOINT, &mut errs) {
                out.push(Destination::url(Path::root().key(key::TRACE).key(key::ENDPOINT), endpoint));
            }
        }
    }
    errs.finish(out)
}
