//! A small MCP server for the tests: `mcp_test_server [legacy] [pid=<file>]`, speaking on its standard
//! streams. Without `legacy` it speaks the current revision of the protocol; with it only the older
//! revision that begins with `initialize`. With `pid=<file>` it writes its process id in the file. Its tools:
//!   say   {text}   answers "said: <text>"
//!   count {}       answers a structured result { "calls": <how many calls this process had> }
//!   boom  {}       answers a result marked as an error
//!   slow  {}       takes ten seconds
//!   die   {}       the first time (marker file named by A_GRAPH_TEST_MARKER) ends the process

use std::sync::atomic::{AtomicU64, Ordering};

use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, DiscoverRequestMethod, DiscoverResult, ErrorData,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServiceExt};
use serde_json::{Value, json};

struct Server {
    legacy: bool,
    calls: AtomicU64,
}

fn tool(name: &'static str, description: &'static str, schema: Value) -> Tool {
    Tool::new(
        name,
        description,
        std::sync::Arc::new(schema.as_object().cloned().unwrap()),
    )
}

impl ServerHandler for Server {
    fn get_info(&self) -> rmcp::model::ServerConfig {
        rmcp::model::ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
    }

    /// An older server does not know `server/discover`.
    async fn discover(&self, _: RequestContext<RoleServer>) -> Result<DiscoverResult, ErrorData> {
        if self.legacy {
            return Err(ErrorData::method_not_found::<DiscoverRequestMethod>());
        }
        Ok(DiscoverResult::from_server_info(
            self.supported_protocol_versions().into_owned(),
            self.get_info(),
        ))
    }

    async fn list_tools(
        &self,
        _: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let none = json!({ "type": "object", "properties": {} });
        let tools = vec![
            tool(
                "say",
                "Says the text back",
                json!({ "type": "object", "properties": { "text": { "type": "string" } }, "required": ["text"] }),
            ),
            tool("count", "Counts the calls of this process", none.clone()),
            tool("boom", "Always fails", none.clone()),
            tool("slow", "Takes ten seconds", none.clone()),
            tool("die", "Ends the process the first time", none),
            tool("odd", "Has a schema that is not valid", json!({ "type": 5 })),
        ];
        Ok(ListToolsResult {
            tools,
            ..Default::default()
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let calls = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        let result = match request.name.as_ref() {
            "say" => {
                let text = request
                    .arguments
                    .as_ref()
                    .and_then(|a| a.get("text"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                CallToolResult::success(vec![rmcp::model::ContentBlock::text(format!("said: {text}"))])
            }
            "count" => CallToolResult::structured(json!({ "calls": calls })),
            "boom" => CallToolResult::error(vec![rmcp::model::ContentBlock::text("it failed")]),
            "slow" => {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                CallToolResult::success(vec![rmcp::model::ContentBlock::text("late")])
            }
            "die" => {
                let marker = std::env::var("A_GRAPH_TEST_MARKER").unwrap_or_default();
                if !std::path::Path::new(&marker).exists() {
                    let _ = std::fs::write(&marker, "died");
                    std::process::exit(1);
                }
                CallToolResult::success(vec![rmcp::model::ContentBlock::text("back")])
            }
            other => return Err(ErrorData::invalid_params(format!("no tool `{other}`"), None)),
        };
        Ok(CallToolResponse::Complete(result))
    }
}

#[tokio::main]
async fn main() {
    let legacy = std::env::args().any(|a| a == "legacy");
    if let Some(file) = std::env::args().find_map(|a| a.strip_prefix("pid=").map(str::to_string)) {
        std::fs::write(file, std::process::id().to_string()).expect("cannot write the pid");
    }
    let service = Server {
        legacy,
        calls: AtomicU64::new(0),
    }
    .serve(rmcp::transport::stdio())
    .await
    .expect("cannot serve");
    let _ = service.waiting().await;
}
