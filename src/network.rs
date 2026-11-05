//! The network permission: every destination the file reaches must be in
//! `passport.adl.permissions.network.allowed_hosts`. Absent `permissions` = nothing is allowed.
//! stdout, files and the verbose display are not network. Hosts, not full URLs: an entry `host`
//! allows every port of the host, an entry `host:port` that port only.
//!
//! The managers say where they go (`destinations()`); this module only compares.

use std::collections::HashSet;

use serde_norway::Value;

use crate::build::{Errs, Obj};
use crate::error::{Error, Errors, Path, Phase, Rule};
use crate::net::{allowed_entry, entry_text, host_port_of};
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
        Destination {
            path,
            target: Target::Url(url.into()),
        }
    }

    pub fn host(path: Path, host: Result<String, String>) -> Self {
        Destination {
            path,
            target: Target::Host(host),
        }
    }
}

/// `passport.adl.permissions.network.allowed_hosts`, lowercase, each `host` or `host:port`.
pub fn allowed_hosts(passport: Option<Value>) -> Result<HashSet<String>, Errors> {
    let mut errs = Errs::default();
    let read = |passport: Value, errs: &mut Errs| -> Option<HashSet<String>> {
        let mut p = Obj::new(passport, Path::root().key(key::PASSPORT), errs)?;
        let mut adl = p.req_sub(key::ADL, errs)?;
        let hosts = adl
            .sub(key::PERMISSIONS, errs)
            .and_then(|mut perm| perm.sub(key::NETWORK, errs))
            .and_then(|mut n| n.opt::<Vec<String>>(key::ALLOWED_HOSTS, errs));
        let at = Path::root()
            .key(key::PASSPORT)
            .key(key::ADL)
            .key(key::PERMISSIONS)
            .key(key::NETWORK)
            .key(key::ALLOWED_HOSTS);
        let mut out = HashSet::new();
        for (i, entry) in hosts.unwrap_or_default().iter().enumerate() {
            match allowed_entry(entry) {
                Ok((host, port)) => _ = out.insert(entry_text(&host, port)),
                Err(why) => errs.add(&at.idx(i), Rule::BadValue, why),
            }
        }
        Some(out)
    };
    let hosts = passport.and_then(|p| read(p, &mut errs));
    errs.finish(hosts.unwrap_or_default())
}

/// Every destination whose host and port are not allowed, or cannot be read, is an error. A
/// provider host is reached on https, port 443.
pub fn check(allowed: &HashSet<String>, destinations: impl IntoIterator<Item = Destination>) -> Vec<Error> {
    let not_allowed = "is not in passport.adl.permissions.network.allowed_hosts";
    let mut errors = Vec::new();
    for d in destinations {
        match host_port(&d.target) {
            Err(why) => errors.push(Error::new(Phase::Interpretation, d.path, Rule::BadValue, why)),
            Ok((h, port)) if !allows(allowed, &h, port) => errors.push(Error::new(
                Phase::Interpretation,
                d.path,
                Rule::NetworkNotAllowed,
                format!("host `{}` {not_allowed}", entry_text(&h, port)),
            )),
            Ok(_) => {}
        }
    }
    errors
}

/// The host and port a target is reached at. A provider host is reached on https, port 443.
pub(crate) fn host_port(target: &Target) -> Result<(String, Option<u16>), String> {
    match target {
        Target::Url(url) => host_port_of(url).ok_or_else(|| format!("`{url}` is not a URL with a host")),
        Target::Host(h) => h.clone().map(|h| (h, Some(443))),
    }
}

/// Whether a list of entries (`host` or `host:port`, as `entry_text` writes them) allows the host
/// on the port: an entry without a port allows every port.
pub(crate) fn allows(entries: &HashSet<String>, host: &str, port: Option<u16>) -> bool {
    entries.contains(host) || port.is_some_and(|p| entries.contains(&entry_text(host, Some(p))))
}

/// The destinations of the logging: `observability.output.url` and `trace.endpoint`.
pub fn logging_destinations(observability: Option<Value>, trace: Option<Value>) -> Result<Vec<Destination>, Errors> {
    let mut errs = Errs::default();
    let mut out = Vec::new();
    let obs_path = Path::root().key(key::OBSERVABILITY).key(key::OUTPUT);
    if let Some(Value::Mapping(mut obs)) = observability {
        // `stdout` is a word and `{ file }` has no url: only `{ url }` is network
        if let Some(Value::Mapping(output)) = obs.remove(key::OUTPUT)
            && let Some(mut o) = Obj::new(Value::Mapping(output), obs_path.clone(), &mut errs)
            && let Some(url) = o.opt::<String>(key::URL, &mut errs)
        {
            out.push(Destination::url(obs_path.key(key::URL), url));
        }
    }
    if let Some(t) = trace
        && let Some(mut t) = Obj::new(t, Path::root().key(key::TRACE), &mut errs)
        && let Some(endpoint) = t.opt::<String>(key::ENDPOINT, &mut errs)
    {
        out.push(Destination::url(
            Path::root().key(key::TRACE).key(key::ENDPOINT),
            endpoint,
        ));
    }
    errs.finish(out)
}
