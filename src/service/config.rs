//! The file that starts the service: its port, where the sessions keep their files, the hosts a
//! graph may reach, how long a run is kept and how many runs go on at once. Its keys are those of
//! `schema::SERVICE`; the first three are mandatory: the service starts only if its owner chose them.

use std::path::PathBuf;

use serde_norway::Value;

use crate::build::{Errs, Obj};
use crate::error::{Errors, Path, Rule};
use crate::net::{allowed_entry, entry_text};
use crate::schema::{SERVICE, key};

/// How long a run is kept after its end when the file does not say.
pub const DEFAULT_RETENTION_SECONDS: u64 = 600;
/// How many runs may go on at once when the file does not say.
pub const DEFAULT_MAX_RUNS: usize = 8;

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub port: u16,
    /// The folder with a subfolder for each session.
    pub data_dir: PathBuf,
    /// The only hosts a graph may reach, lowercase, each `host` or `host:port`.
    pub allowed_hosts: Vec<String>,
    /// Seconds a run is kept after it ends (so it can still be watched and asked for).
    pub retention_seconds: u64,
    /// The most runs going on at once; a request beyond it is refused.
    pub max_runs: usize,
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
        let Some(mut o) = Obj::new(root, Path::root(), &mut errs) else { return Err(Errors(errs.0)) };
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
        let retention = o.opt::<u64>(key::RETENTION_SECONDS, &mut errs);
        if retention == Some(0) {
            errs.add(&Path::root().key(key::RETENTION_SECONDS), Rule::BadValue, "`retention_seconds` is at least 1");
        }
        // u32: a number the semaphore can hold
        let max_runs = o.opt::<u32>(key::MAX_RUNS, &mut errs);
        if max_runs == Some(0) {
            errs.add(&Path::root().key(key::MAX_RUNS), Rule::BadValue, "`max_runs` is at least 1");
        }
        match (port, data_dir, hosts, errs.0.is_empty()) {
            (Some(port), Some(dir), Some(hosts), true) => Ok(ServiceConfig { port, data_dir: PathBuf::from(dir), allowed_hosts: hosts, retention_seconds: retention.unwrap_or(DEFAULT_RETENTION_SECONDS), max_runs: max_runs.map_or(DEFAULT_MAX_RUNS, |n| n as usize) }),
            _ => Err(Errors(errs.0)),
        }
    }
}
