//! `resources.mcp_servers`: external servers that speak the MCP protocol.

use std::collections::BTreeMap;

use serde_norway::Value;

use super::common::{Errs, Obj, RawTransport, list_path};
use crate::error::{Errors, Rule};
use crate::network::Destination;
use crate::schema::key;

/// A secret is kept as the name of the environment variable, never as its value.
#[derive(Debug, Clone)]
pub enum McpTransport {
    Stdio {
        command: String,
        args: Vec<String>,
        env: BTreeMap<String, String>,
    },
    Http {
        url: String,
        headers: BTreeMap<String, String>,
    },
}

#[derive(Debug, Clone)]
pub struct McpServer {
    pub id: String,
    pub transport: McpTransport,
    /// The most calls to the tools of this server that one run of a node may make.
    pub max_iter: u32,
    /// Seconds one call may take; after that the call has failed.
    pub timeout: u32,
}

#[derive(Debug, Default)]
pub struct McpManager {
    servers: Vec<McpServer>,
}

impl McpManager {
    /// Builds from the value of `resources.mcp_servers`.
    pub fn build(list: Option<Value>) -> Result<Self, Errors> {
        let mut errs = Errs::default();
        let mut servers = Vec::new();
        for mut o in Obj::items(&list_path(key::MCP_SERVERS), list, &mut errs) {
            let before = errs.0.len();
            let at = o.at().clone();
            let id = o.req::<String>(key::ID, &mut errs);
            let kind = o.req::<RawTransport>(key::TRANSPORT, &mut errs);
            let command = o.opt::<String>(key::COMMAND, &mut errs);
            let args = o.opt::<Vec<String>>(key::ARGS, &mut errs).unwrap_or_default();
            let env = o.secrets(key::ENV, &mut errs);
            let url = o.opt::<String>(key::URL, &mut errs);
            let headers = o.secrets(key::HEADERS, &mut errs);
            let max_iter = o.req::<u32>(key::MAX_ITER, &mut errs);
            let timeout = o.req::<u32>(key::TIMEOUT, &mut errs);
            if max_iter == Some(0) {
                errs.add(&at.key(key::MAX_ITER), Rule::BadValue, "`max_iter` is at least 1");
            }
            if timeout == Some(0) {
                errs.add(&at.key(key::TIMEOUT), Rule::BadValue, "`timeout` is at least 1 second");
            }

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
                    command.map(|command| McpTransport::Stdio { command, args, env })
                }
                Some(RawTransport::Http) => {
                    errs.require(&at, key::URL, url.is_some(), "transport http needs `url`");
                    errs.forbid(&at, key::COMMAND, command.is_some(), "`command` is for transport stdio");
                    errs.forbid(&at, key::ARGS, !args.is_empty(), "`args` is for transport stdio");
                    errs.forbid(&at, key::ENV, !env.is_empty(), "`env` is for transport stdio");
                    url.map(|url| McpTransport::Http { url, headers })
                }
                None => None,
            };
            if let (Some(id), Some(transport), Some(max_iter), Some(timeout), true) =
                (id, transport, max_iter, timeout, errs.0.len() == before)
            {
                servers.push(McpServer {
                    id,
                    transport,
                    max_iter,
                    timeout,
                });
            }
        }
        errs.finish(McpManager { servers })
    }

    /// The urls of the http servers, for the network permission.
    pub fn destinations(&self) -> Vec<Destination> {
        let base = list_path(key::MCP_SERVERS);
        self.servers
            .iter()
            .enumerate()
            .filter_map(|(i, s)| match &s.transport {
                McpTransport::Http { url, .. } => Some(Destination::url(base.idx(i).key(key::URL), url)),
                McpTransport::Stdio { .. } => None,
            })
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<&McpServer> {
        self.servers.iter().find(|s| s.id == id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &McpServer> {
        self.servers.iter()
    }
}
