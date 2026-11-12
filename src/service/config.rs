//! The file that starts the service: its port, where the sessions keep their files, the hosts a
//! graph may reach, how long a run is kept, how many go on at once and how long one lasts, how long
//! the body of a request may take to arrive, and how
//! long a session is kept and how much it may weigh, the environment variables a graph may use and
//! the providers that may use the credentials of the service. Its keys are those of
//! `schema::SERVICE`; the first five are mandatory: the service starts only if its owner chose them.

use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

use serde_norway::Value;

use crate::build::{Errs, Obj};
use crate::error::{Errors, Path, Rule};
use crate::net::{allowed_entry, entry_text};
use crate::network::allows;
use crate::schema::{SERVICE, key};
use crate::secrets::AllowedEnv;

/// How long a run is kept after its end when the file does not say.
pub const DEFAULT_RETENTION_SECONDS: u64 = 600;
/// How many runs may go on at once when the file does not say.
pub const DEFAULT_MAX_RUNS: usize = 8;
/// The longest a run may last when the file does not say: an hour.
pub const DEFAULT_RUN_TIMEOUT_SECONDS: u64 = 3600;
/// The longest the body of a request may take to arrive when the file does not say.
pub const DEFAULT_READ_TIMEOUT_SECONDS: u64 = 60;

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub port: u16,
    /// The folder with a subfolder for each session.
    pub data_dir: PathBuf,
    /// The only hosts a graph may reach, lowercase, each `host` or `host:port`.
    pub allowed_hosts: Vec<String>,
    /// The environment variables a graph may use, each with the hosts its value may go to.
    pub allowed_env: AllowedEnv,
    /// The providers whose models may use the credentials of the service (`credentials: ambient`).
    pub allowed_ambient: Vec<String>,
    /// Seconds a run is kept after it ends (so it can still be watched and asked for).
    pub retention_seconds: u64,
    /// The most runs going on at once; a request beyond it is refused.
    pub max_runs: usize,
    /// Seconds a run may last; then it ends as failed (`run_timeout`).
    pub run_timeout_seconds: u64,
    /// Seconds the body of a request may take to arrive; then it is refused (`read_timeout`).
    pub read_timeout_seconds: u64,
    /// Seconds a session folder is kept after its last request; `None`: never deleted.
    pub session_retention_seconds: Option<u64>,
    /// The most a session folder may weigh before a request for it is refused; `None`: no cap.
    pub session_max_bytes: Option<u64>,
}

impl ServiceConfig {
    /// Reads the text of the file. Every error is reported, not only the first.
    pub fn read(src: &str) -> Result<Self, Errors> {
        let mut errs = Errs::default();
        let root: Value = match serde_norway::from_str(src) {
            Ok(v) => v,
            Err(e) => {
                errs.add(&Path::root(), Rule::Parse, e.to_string());
                return Err(Errors(errs.0));
            }
        };
        // the keys and the kinds of their values, from the table of `schema`
        let shape = crate::loader::check_shape(&root, SERVICE);
        if !shape.is_empty() {
            return Err(Errors(shape));
        }
        let Some(mut o) = Obj::new(root, Path::root(), &mut errs) else {
            return Err(Errors(errs.0));
        };
        let port = o.req::<u16>(key::PORT, &mut errs);
        let data_dir = o.req::<String>(key::DATA_DIR, &mut errs);
        let hosts = o.req::<Vec<String>>(key::ALLOWED_HOSTS, &mut errs).map(|hosts| {
            let mut out = Vec::new();
            for (i, entry) in hosts.iter().enumerate() {
                match allowed_entry(entry) {
                    Ok((host, port)) => out.push(entry_text(&host, port)),
                    Err(why) => errs.add(&Path::root().key(key::ALLOWED_HOSTS).idx(i), Rule::BadValue, why),
                }
            }
            out
        });
        let env = o
            .req::<BTreeMap<String, Vec<String>>>(key::ALLOWED_ENV, &mut errs)
            .map(|env| allowed_env(env, hosts.as_deref().unwrap_or_default(), &mut errs));
        let ambient = o.req::<Vec<String>>(key::ALLOWED_AMBIENT, &mut errs);
        let retention = o.opt::<u64>(key::RETENTION_SECONDS, &mut errs);
        if retention == Some(0) {
            errs.add(
                &Path::root().key(key::RETENTION_SECONDS),
                Rule::BadValue,
                "`retention_seconds` is at least 1",
            );
        }
        // u32: a number the semaphore can hold
        let max_runs = o.opt::<u32>(key::MAX_RUNS, &mut errs);
        if max_runs == Some(0) {
            errs.add(
                &Path::root().key(key::MAX_RUNS),
                Rule::BadValue,
                "`max_runs` is at least 1",
            );
        }
        let run_timeout = o.opt::<u64>(key::RUN_TIMEOUT_SECONDS, &mut errs);
        if run_timeout == Some(0) {
            errs.add(
                &Path::root().key(key::RUN_TIMEOUT_SECONDS),
                Rule::BadValue,
                "`run_timeout_seconds` is at least 1",
            );
        }
        let read_timeout = o.opt::<u64>(key::READ_TIMEOUT_SECONDS, &mut errs);
        if read_timeout == Some(0) {
            errs.add(
                &Path::root().key(key::READ_TIMEOUT_SECONDS),
                Rule::BadValue,
                "`read_timeout_seconds` is at least 1",
            );
        }
        let session_retention = o.opt::<u64>(key::SESSION_RETENTION_SECONDS, &mut errs);
        if session_retention == Some(0) {
            errs.add(
                &Path::root().key(key::SESSION_RETENTION_SECONDS),
                Rule::BadValue,
                "`session_retention_seconds` is at least 1",
            );
        }
        let session_max_bytes = o.opt::<u64>(key::SESSION_MAX_BYTES, &mut errs);
        if session_max_bytes == Some(0) {
            errs.add(
                &Path::root().key(key::SESSION_MAX_BYTES),
                Rule::BadValue,
                "`session_max_bytes` is at least 1",
            );
        }
        match (port, data_dir, hosts, env, ambient, errs.0.is_empty()) {
            (Some(port), Some(dir), Some(hosts), Some(env), Some(ambient), true) => Ok(ServiceConfig {
                port,
                data_dir: PathBuf::from(dir),
                allowed_hosts: hosts,
                allowed_env: env,
                allowed_ambient: ambient,
                retention_seconds: retention.unwrap_or(DEFAULT_RETENTION_SECONDS),
                max_runs: max_runs.map_or(DEFAULT_MAX_RUNS, |n| n as usize),
                run_timeout_seconds: run_timeout.unwrap_or(DEFAULT_RUN_TIMEOUT_SECONDS),
                read_timeout_seconds: read_timeout.unwrap_or(DEFAULT_READ_TIMEOUT_SECONDS),
                session_retention_seconds: session_retention,
                session_max_bytes,
            }),
            _ => Err(Errors(errs.0)),
        }
    }
}

/// `allowed_env`, each host read as an entry of `allowed_hosts` is. A variable without hosts, or
/// with a host that `allowed_hosts` does not allow, is an error: no graph could send its value.
fn allowed_env(env: BTreeMap<String, Vec<String>>, allowed_hosts: &[String], errs: &mut Errs) -> AllowedEnv {
    let service: HashSet<String> = allowed_hosts.iter().cloned().collect();
    let mut out = BTreeMap::new();
    for (var, hosts) in env {
        let at = Path::root().key(key::ALLOWED_ENV).key(&var);
        if hosts.is_empty() {
            errs.add(
                &at,
                Rule::BadValue,
                format!("`{var}` lists no host: no graph could use it"),
            );
        }
        let mut entries = HashSet::new();
        for (i, entry) in hosts.iter().enumerate() {
            match allowed_entry(entry) {
                Err(why) => errs.add(&at.idx(i), Rule::BadValue, why),
                Ok((host, port)) if !allows(&service, &host, port) => errs.add(
                    &at.idx(i),
                    Rule::NetworkNotAllowed,
                    format!("host `{entry}` is not in allowed_hosts: no graph could send `{var}` there"),
                ),
                Ok((host, port)) => _ = entries.insert(entry_text(&host, port)),
            }
        }
        out.insert(var, entries);
    }
    AllowedEnv(out)
}
