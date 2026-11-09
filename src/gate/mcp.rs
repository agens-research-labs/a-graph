//! The MCP servers of a run: they are connected once, when the run starts, only the ones some node
//! uses, and kept open until it ends. The protocol is spoken by `rmcp`, the official SDK, which
//! tries the current revision (`server/discover`, no handshake) and falls back to the older ones
//! (`initialize`). Only the tools of a server are used.

use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use rmcp::RoleClient;
use rmcp::model::{CallToolRequestParams, ContentBlock, ProtocolVersion};
use rmcp::service::{ClientLifecycleMode, ClientServiceExt, RunningService};
use rmcp::transport::{
    StreamableHttpClientTransport, TokioChildProcess, streamable_http_client::StreamableHttpClientTransportConfig,
};
use serde_json::Value;
use tokio::process::Command;
use tokio::sync::Mutex;

use super::ToolOutcome;
use crate::models::ToolDefinition;
use crate::resources::{McpServer, McpTransport};

type Service = RunningService<RoleClient, ()>;

/// One server: how to reach it, the tools it gave when it connected, and the open connection.
struct Connection {
    server: McpServer,
    tools: Vec<ToolDefinition>,
    /// `None` when the connection was lost and could not be made again.
    service: Mutex<Option<Service>>,
}

/// The servers a run uses, connected.
#[derive(Default)]
pub struct McpPool {
    connections: HashMap<String, Connection>,
}

impl McpPool {
    /// Connects every server in `servers` and asks each for its tools. The error says which server
    /// could not be reached and why.
    pub async fn connect(servers: &[McpServer]) -> Result<McpPool, String> {
        let mut connections = HashMap::new();
        for server in servers {
            let service = open(server)
                .await
                .map_err(|e| format!("MCP server `{}`: {e}", server.id))?;
            let tools = definitions(&service)
                .await
                .map_err(|e| format!("MCP server `{}`: {e}", server.id))?;
            connections.insert(
                server.id.clone(),
                Connection {
                    server: server.clone(),
                    tools,
                    service: Mutex::new(Some(service)),
                },
            );
        }
        Ok(McpPool { connections })
    }

    /// The tools a server gave when it connected.
    pub fn tools(&self, server: &str) -> &[ToolDefinition] {
        self.connections
            .get(server)
            .map(|c| c.tools.as_slice())
            .unwrap_or_default()
    }

    pub fn is_connected(&self, server: &str) -> bool {
        self.connections.contains_key(server)
    }

    /// Calls a tool of a server. If the connection was lost it is made again once and the call is
    /// repeated; if that fails too, the call has failed and the model is told. `Err` is only a
    /// server that is not in the pool.
    pub async fn call(&self, server: &str, tool: &str, arguments: &Value) -> Result<ToolOutcome, String> {
        let c = self
            .connections
            .get(server)
            .ok_or_else(|| format!("MCP server `{server}` is not connected"))?;
        let arguments = arguments.as_object().cloned();
        let limit = Duration::from_secs(c.server.timeout.into());
        let attempt = async {
            match try_call(c, tool, arguments.clone()).await {
                Err(Lost) => {
                    // the server went away: connect again once and repeat the call
                    let mut guard = c.service.lock().await;
                    *guard = open(&c.server).await.ok();
                    drop(guard);
                    try_call(c, tool, arguments)
                        .await
                        .unwrap_or_else(|Lost| ToolOutcome::failed(format!("MCP server `{server}` is not reachable")))
                }
                Ok(outcome) => outcome,
            }
        };
        match tokio::time::timeout(limit, attempt).await {
            Ok(outcome) => Ok(outcome),
            Err(_) => Ok(ToolOutcome::failed(format!(
                "the tool did not answer within {} s",
                c.server.timeout
            ))),
        }
    }

    /// Closes every connection: a server process gets its input closed and is stopped.
    pub async fn close(self) {
        for (_, c) in self.connections {
            if let Some(mut service) = c.service.into_inner() {
                // a server that is busy does not hold the end of the run: it is let go after a moment
                let _ = service.close_with_timeout(Duration::from_secs(2)).await;
            }
        }
    }
}

/// The connection was lost, not the tool failed.
struct Lost;

async fn try_call(
    c: &Connection,
    tool: &str,
    arguments: Option<serde_json::Map<String, Value>>,
) -> Result<ToolOutcome, Lost> {
    let guard = c.service.lock().await;
    let service = guard.as_ref().ok_or(Lost)?;
    let mut params = CallToolRequestParams::new(tool.to_string());
    params.arguments = arguments;
    match service.call_tool(params).await {
        Ok(result) => {
            let structured = result.structured_content.clone();
            let value = structured.unwrap_or_else(|| Value::String(text_of(&result.content)));
            Ok(ToolOutcome {
                result: value,
                is_error: result.is_error == Some(true),
                without_data: None,
            })
        }
        Err(rmcp::ServiceError::McpError(e)) => Ok(ToolOutcome::failed(e.message.to_string())),
        Err(rmcp::ServiceError::TransportClosed | rmcp::ServiceError::TransportSend(_)) => Err(Lost),
        Err(e) => Ok(ToolOutcome::failed(e.to_string())),
    }
}

/// The text blocks of a result, one after the other; any other kind of block as JSON.
fn text_of(content: &[ContentBlock]) -> String {
    content
        .iter()
        .map(|b| match b {
            ContentBlock::Text(t) => t.text.clone(),
            other => serde_json::to_string(other).unwrap_or_default(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

async fn definitions(service: &Service) -> Result<Vec<ToolDefinition>, String> {
    let tools = service
        .list_all_tools()
        .await
        .map_err(|e| format!("cannot list the tools: {e}"))?;
    Ok(tools
        .into_iter()
        .map(|t| ToolDefinition {
            name: t.name.to_string(),
            description: t.description.map(|d| d.to_string()).unwrap_or_default(),
            parameters: Value::Object((*t.input_schema).clone()),
        })
        .collect())
}

/// Opens the connection: a program for `stdio`, a web service for `http`.
async fn open(server: &McpServer) -> Result<Service, String> {
    let versions = vec![ProtocolVersion::LATEST];
    let mode = ClientLifecycleMode::Auto {
        preferred_versions: versions,
        legacy_version: None,
    };
    match &server.transport {
        McpTransport::Stdio { command, args, env } => {
            let mut cmd = Command::new(command);
            cmd.args(args);
            for (name, var) in env {
                cmd.env(
                    name,
                    std::env::var(var).map_err(|_| format!("environment variable `{var}` is not set"))?,
                );
            }
            let transport = TokioChildProcess::new(cmd).map_err(|e| format!("cannot start `{command}`: {e}"))?;
            ().serve_with_lifecycle(transport, mode)
                .await
                .map_err(|e| format!("cannot connect: {e}"))
        }
        McpTransport::Http { url, headers } => {
            let mut config = StreamableHttpClientTransportConfig::with_uri(url.as_str());
            config.custom_headers = header_map(headers)?;
            let client = crate::net::client()
                .build()
                .map_err(|e| format!("cannot build the HTTP client: {e}"))?;
            let transport = StreamableHttpClientTransport::with_client(client, config);
            ().serve_with_lifecycle(transport, mode)
                .await
                .map_err(|e| format!("cannot connect: {e}"))
        }
    }
}

fn header_map(headers: &BTreeMap<String, String>) -> Result<HashMap<http::HeaderName, http::HeaderValue>, String> {
    let mut out = HashMap::new();
    for (name, var) in headers {
        let value = std::env::var(var).map_err(|_| format!("environment variable `{var}` is not set"))?;
        let name = http::HeaderName::try_from(name.as_str()).map_err(|e| format!("header `{name}`: {e}"))?;
        out.insert(
            name,
            crate::net::secret_value(&value).map_err(|e| format!("a header value is not valid: {e}"))?,
        );
    }
    Ok(out)
}
