//! The gate: the only way a node reaches a tool. The model asks for a call, the node hands it to
//! the gate, the gate talks to the tool (a program or a web service, in any language) and gives the
//! answer back. The node and the model never talk to the tool directly.
//!
//! The contract of a tool: it receives one JSON object, the arguments the model chose, and answers
//! one JSON value, the result. A program (`stdio`) is started for each call, gets the arguments on
//! its standard input and writes the result on its standard output; if it exits with a code other
//! than 0 the call failed and the text it wrote on its standard error is the result. A web service
//! (`http`) gets the arguments as the body of a POST and answers the result as the body; any status
//! other than 2xx means the call failed and the body is the result. A call that takes longer than
//! the `timeout` of the tool has failed too.

mod mcp;
mod script;
mod tool;

pub use mcp::McpPool;
pub use script::run_script;
pub use tool::call_tool;
pub(crate) use tool::{http as http_call, stdio as stdio_call};

use serde_json::Value;

/// What a tool gave back, for the model: a result, or the reason the call failed.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolOutcome {
    pub result: Value,
    pub is_error: bool,
}

impl ToolOutcome {
    pub(crate) fn ok(result: Value) -> Self {
        ToolOutcome {
            result,
            is_error: false,
        }
    }

    pub(crate) fn failed(why: impl Into<String>) -> Self {
        ToolOutcome {
            result: Value::String(why.into()),
            is_error: true,
        }
    }
}
