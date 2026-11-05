//! The environment variables a graph sent to the service may use: every `{ env: VAR }` of the
//! graph must be in the `allowed_env` of the service, with every host its value goes to. A graph
//! run from the command line is not checked: the one who writes it runs it, with their own
//! variables.
//!
//! The managers say which variables they use and where each value goes (`secrets()`); this
//! module only compares. Nothing here reads a variable.

use std::collections::{BTreeMap, HashSet};

use serde_norway::Value;

use crate::build::{Errs, Obj};
use crate::error::{Error, Errors, Path, Phase, Rule};
use crate::network::{Destination, allows, host_port};
use crate::schema::key;

/// A `{ env: VAR }` of the yml: where it is written, the variable, and the destinations its value
/// is sent to. No destination: the value is given to a program, not sent to a host.
#[derive(Debug, Clone)]
pub struct Secret {
    pub path: Path,
    pub var: String,
    pub destinations: Vec<Destination>,
}

/// The `allowed_env` of the service: each variable a graph may use, with the hosts its value may
/// go to, each `host` or `host:port` as `allowed_hosts` writes them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AllowedEnv(pub BTreeMap<String, HashSet<String>>);

/// Every secret whose variable is not allowed, or whose value would go to a host its variable is
/// not allowed to reach, is an error. A destination whose host cannot be read is left to the
/// check of the hosts, which reports it.
pub fn check(allowed: &AllowedEnv, secrets: impl IntoIterator<Item = Secret>) -> Vec<Error> {
    let mut errors = Vec::new();
    for s in secrets {
        let refuse = |msg: String| Error::new(Phase::Interpretation, s.path.clone(), Rule::EnvNotAllowed, msg);
        let Some(hosts) = allowed.0.get(&s.var) else {
            errors.push(refuse(format!(
                "environment variable `{}` is not in the allowed_env of this service",
                s.var
            )));
            continue;
        };
        if s.destinations.is_empty() {
            errors.push(refuse(format!(
                "environment variable `{}` is given to a program, not sent to a host of its allowed_env",
                s.var
            )));
        }
        for d in &s.destinations {
            if let Ok((host, port)) = host_port(&d.target)
                && !allows(hosts, &host, port)
            {
                errors.push(refuse(format!(
                    "environment variable `{}` may not be sent to `{}`: it is not in its allowed_env",
                    s.var,
                    crate::net::entry_text(&host, port)
                )));
            }
        }
    }
    errors
}

/// The secrets of the log: the headers of `observability.output`, sent to its `url`.
pub fn log_secrets(observability: Option<Value>) -> Result<Vec<Secret>, Errors> {
    let at = Path::root().key(key::OBSERVABILITY).key(key::OUTPUT);
    let mut errs = Errs::default();
    let mut out = Vec::new();
    if let Some(Value::Mapping(mut obs)) = observability
        && let Some(output @ Value::Mapping(_)) = obs.remove(key::OUTPUT)
        && let Some(mut o) = Obj::new(output, at.clone(), &mut errs)
        && let Some(url) = o.opt::<String>(key::URL, &mut errs)
    {
        let to = Destination::url(at.key(key::URL), url);
        out = headers(&o.secrets(key::HEADERS, &mut errs), &at, &to);
    }
    errs.finish(out)
}

/// The secrets of the trace: the headers of `trace`, sent to its `endpoint`.
pub fn trace_secrets(trace: Option<Value>) -> Result<Vec<Secret>, Errors> {
    let at = Path::root().key(key::TRACE);
    let mut errs = Errs::default();
    let mut out = Vec::new();
    if let Some(t) = trace
        && let Some(mut t) = Obj::new(t, at.clone(), &mut errs)
        && let Some(endpoint) = t.opt::<String>(key::ENDPOINT, &mut errs)
    {
        let to = Destination::url(at.key(key::ENDPOINT), endpoint);
        out = headers(&t.secrets(key::HEADERS, &mut errs), &at, &to);
    }
    errs.finish(out)
}

/// One secret for each header of `at`, all sent to the same destination.
pub(crate) fn headers(headers: &BTreeMap<String, String>, at: &Path, to: &Destination) -> Vec<Secret> {
    headers
        .iter()
        .map(|(name, var)| Secret {
            path: at.key(key::HEADERS).key(name),
            var: var.clone(),
            destinations: vec![to.clone()],
        })
        .collect()
}
