//! `resources.tools`: a-graph's own minimal request/response tools.

use std::collections::BTreeMap;

use serde_norway::Value;

use super::common::{Errs, Obj, RawTransport, list_path};
use crate::error::{Errors, Rule};
use crate::network::Destination;
use crate::schema::key;

#[derive(Debug, Clone)]
pub enum ToolTransport {
    Stdio { command: String, args: Vec<String> },
    /// The headers are kept as the name of the environment variable that holds each value.
    Http { url: String, headers: BTreeMap<String, String> },
}

#[derive(Debug, Clone)]
pub struct Tool {
    pub id: String,
    pub description: String,
    pub transport: ToolTransport,
    /// The most calls of this tool that one run of a node may make.
    pub max_iter: u32,
    /// Seconds one call may take; after that the call has failed.
    pub timeout: u32,
    /// Raw JSON Schema, passed to the provider as written.
    pub parameters: Value,
}

#[derive(Debug, Default)]
pub struct ToolsManager {
    tools: Vec<Tool>,
}

impl ToolsManager {
    /// Builds from the value of `resources.tools`.
    pub fn build(list: Option<Value>) -> Result<Self, Errors> {
        let mut errs = Errs::default();
        let mut tools = Vec::new();
        for mut o in Obj::items(&list_path(key::TOOLS), list, &mut errs) {
            let before = errs.0.len();
            let at = o.at().clone();
            let id = o.req::<String>(key::ID, &mut errs);
            let description = o.req::<String>(key::DESCRIPTION, &mut errs);
            let kind = o.req::<RawTransport>(key::TRANSPORT, &mut errs);
            let command = o.opt::<String>(key::COMMAND, &mut errs);
            let args = o.opt::<Vec<String>>(key::ARGS, &mut errs).unwrap_or_default();
            let url = o.opt::<String>(key::URL, &mut errs);
            let headers = o.secrets(key::HEADERS, &mut errs);
            let max_iter = o.req::<u32>(key::MAX_ITER, &mut errs);
            let timeout = o.req::<u32>(key::TIMEOUT, &mut errs);
            let parameters = o.value(key::PARAMETERS);

            let transport = match kind {
                Some(RawTransport::Stdio) => {
                    errs.require(&at, key::COMMAND, command.is_some(), "transport stdio needs `command`");
                    errs.forbid(&at, key::URL, url.is_some(), "`url` is for transport http");
                    errs.forbid(&at, key::HEADERS, !headers.is_empty(), "`headers` is for transport http");
                    command.map(|command| ToolTransport::Stdio { command, args })
                }
                Some(RawTransport::Http) => {
                    errs.require(&at, key::URL, url.is_some(), "transport http needs `url`");
                    errs.forbid(&at, key::COMMAND, command.is_some(), "`command` is for transport stdio");
                    errs.forbid(&at, key::ARGS, !args.is_empty(), "`args` is for transport stdio");
                    for (name, var) in &headers {
                        if std::env::var(var).is_err() {
                            errs.add(&at.key(key::HEADERS).key(name), Rule::EnvMissing, format!("environment variable `{var}` is not set"));
                        }
                    }
                    url.map(|url| ToolTransport::Http { url, headers })
                }
                None => None,
            };
            if max_iter == Some(0) {
                errs.add(&at.key(key::MAX_ITER), Rule::BadValue, "`max_iter` is at least 1");
            }
            if timeout == Some(0) {
                errs.add(&at.key(key::TIMEOUT), Rule::BadValue, "`timeout` is at least 1 second");
            }
            let parameters = match parameters {
                Some(p) if p.is_mapping() => match serde_json::to_value(&p).map(|j| jsonschema::validator_for(&j)) {
                    Ok(Ok(_)) => Some(p),
                    Ok(Err(e)) => {
                        errs.add(&at.key(key::PARAMETERS), Rule::BadValue, format!("parameters is not a valid JSON Schema: {e}"));
                        None
                    }
                    Err(_) => None,
                },
                _ => {
                    errs.add(&at.key(key::PARAMETERS), Rule::BadValue, "parameters must be a JSON Schema object");
                    None
                }
            };
            if let (Some(id), Some(description), Some(transport), Some(max_iter), Some(timeout), Some(parameters), true) =
                (id, description, transport, max_iter, timeout, parameters, errs.0.len() == before)
            {
                tools.push(Tool { id, description, transport, max_iter, timeout, parameters });
            }
        }
        errs.finish(ToolsManager { tools })
    }

    /// The urls of the http tools, for the network permission.
    pub fn destinations(&self) -> Vec<Destination> {
        let base = list_path(key::TOOLS);
        self.tools
            .iter()
            .enumerate()
            .filter_map(|(i, t)| match &t.transport {
                ToolTransport::Http { url, .. } => Some(Destination::url(base.idx(i).key(key::URL), url)),
                ToolTransport::Stdio { .. } => None,
            })
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<&Tool> {
        self.tools.iter().find(|t| t.id == id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Tool> {
        self.tools.iter()
    }
}
