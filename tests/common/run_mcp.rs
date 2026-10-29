//! The MCP servers. Every test is a folder with its own graph; the model is a mock
//! server (on a free port) like in the tools, and the MCP server is the project
//! `tests/mcp_server`, which the tests build when they need it.

pub use std::sync::{Arc, Mutex};
pub use serde_json::{Value, json};
pub use a_graph::run::{Program, build};
pub use super::run::{yml_in, yml_with};
pub use super::http::{Req, Resp, listen};
pub use super::toolmock::{body_of, city, items, model};

pub async fn program(dir: &std::path::Path, name: &str, handler: impl Fn(&Req) -> Resp + Send + Sync + 'static) -> (Arc<Program>, Arc<Mutex<Vec<Req>>>) {
    super::ensure_mcp_server();
    let (port, seen) = listen(handler).await;
    let yml = yml_with(dir, name, &[("model", port)]);
    (Arc::new(build(&yml).unwrap_or_else(|e| panic!("{e}"))), seen)
}

pub fn ask(name: &'static str, args: Value) -> Vec<(&'static str, Value)> {
    vec![(name, args)]
}











