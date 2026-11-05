//! The `utilities` block: what the yml says about each utility.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_norway::Value;

use crate::build::{Errs, Obj};
use crate::error::{Error, Errors, Path, Rule};
use crate::network::Destination;
use crate::schema::key;

/// The value of `transport`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum RawTransport {
    Stdio,
    Http,
}

#[derive(Debug, Clone)]
pub enum UtilityTransport {
    /// A program, started for each call.
    Stdio { command: String, args: Vec<String> },
    /// A service already running. The headers are kept as the name of the variable holding each value.
    Http {
        url: String,
        headers: BTreeMap<String, String>,
    },
}

#[derive(Debug, Clone)]
pub struct Utility {
    pub id: String,
    pub transport: UtilityTransport,
    /// Seconds one call may take; after that the call has failed.
    pub timeout: u32,
}

#[derive(Debug, Default)]
pub struct UtilityManager {
    utilities: Vec<Utility>,
}

impl UtilityManager {
    /// Builds from the value of the `utilities` block, already validated by the Loader.
    pub fn build(list: Option<Value>) -> Result<Self, Errors> {
        let mut errs = Errs::default();
        let mut utilities = Vec::new();
        for mut o in Obj::items(&Path::root().key(key::UTILITIES), list, &mut errs) {
            let before = errs.0.len();
            let at = o.at().clone();
            let id = o.req::<String>(key::ID, &mut errs);
            let kind = o.req::<RawTransport>(key::TRANSPORT, &mut errs);
            let command = o.opt::<String>(key::COMMAND, &mut errs);
            let args = o.opt::<Vec<String>>(key::ARGS, &mut errs).unwrap_or_default();
            let url = o.opt::<String>(key::URL, &mut errs);
            let headers = o.secrets(key::HEADERS, &mut errs);
            let timeout = o.req::<u32>(key::TIMEOUT, &mut errs);

            let transport = match kind {
                Some(RawTransport::Stdio) => {
                    errs.require(&at, key::COMMAND, command.is_some(), "transport stdio needs `command`");
                    errs.forbid(&at, key::URL, url.is_some(), "`url` is for transport http");
                    errs.forbid(
                        &at,
                        key::HEADERS,
                        !headers.is_empty(),
                        "`headers` is for transport http",
                    );
                    command.map(|command| UtilityTransport::Stdio { command, args })
                }
                Some(RawTransport::Http) => {
                    errs.require(&at, key::URL, url.is_some(), "transport http needs `url`");
                    errs.forbid(&at, key::COMMAND, command.is_some(), "`command` is for transport stdio");
                    errs.forbid(&at, key::ARGS, !args.is_empty(), "`args` is for transport stdio");
                    url.map(|url| UtilityTransport::Http { url, headers })
                }
                None => None,
            };
            if timeout == Some(0) {
                errs.add(&at.key(key::TIMEOUT), Rule::BadValue, "`timeout` is at least 1 second");
            }
            if let (Some(id), Some(transport), Some(timeout), true) = (id, transport, timeout, errs.0.len() == before) {
                utilities.push(Utility { id, transport, timeout });
            }
        }
        errs.finish(UtilityManager { utilities })
    }

    /// The urls of the http utilities, for the network permission.
    pub fn destinations(&self) -> Vec<Destination> {
        let base = Path::root().key(key::UTILITIES);
        self.utilities
            .iter()
            .enumerate()
            .filter_map(|(i, u)| match &u.transport {
                UtilityTransport::Http { url, .. } => Some(Destination::url(base.idx(i).key(key::URL), url)),
                UtilityTransport::Stdio { .. } => None,
            })
            .collect()
    }

    /// A header whose environment variable is not set, for every http utility: read after the
    /// checks of the build, the values are read again at each call.
    pub fn missing_env(&self) -> Vec<Error> {
        let base = Path::root().key(key::UTILITIES);
        let mut errs = Errs::default();
        for (i, u) in self.utilities.iter().enumerate() {
            if let UtilityTransport::Http { headers, .. } = &u.transport {
                for (name, var) in headers {
                    if std::env::var(var).is_err() {
                        errs.add(
                            &base.idx(i).key(key::HEADERS).key(name),
                            Rule::EnvMissing,
                            format!("environment variable `{var}` is not set"),
                        );
                    }
                }
            }
        }
        errs.0
    }

    pub fn get(&self, id: &str) -> Option<&Utility> {
        self.utilities.iter().find(|u| u.id == id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Utility> {
        self.utilities.iter()
    }

    pub fn len(&self) -> usize {
        self.utilities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.utilities.is_empty()
    }
}
