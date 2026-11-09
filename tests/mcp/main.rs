//! Tests of the topic `mcp`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "call_timeout/call_timeout.rs"]
mod call_timeout;

#[path = "cannot_connect/cannot_connect.rs"]
mod cannot_connect;

#[path = "current_revision/current_revision.rs"]
mod current_revision;

#[path = "error_result/error_result.rs"]
mod error_result;

#[path = "http_body_limit/http_body_limit.rs"]
mod http_body_limit;

#[path = "mcp_alone_refused/mcp_alone_refused.rs"]
mod mcp_alone_refused;

#[path = "mcp_env_headers/mcp_env_headers.rs"]
mod mcp_env_headers;

#[path = "older_revision/older_revision.rs"]
mod older_revision;

#[path = "one_process_per_run/one_process_per_run.rs"]
mod one_process_per_run;

#[path = "over_max_iter/over_max_iter.rs"]
mod over_max_iter;

#[path = "own_and_server_tools/own_and_server_tools.rs"]
mod own_and_server_tools;

#[path = "restart_and_retry/restart_and_retry.rs"]
mod restart_and_retry;

#[path = "same_tool_name/same_tool_name.rs"]
mod same_tool_name;

#[path = "stopped_closes/stopped_closes.rs"]
mod stopped_closes;

#[path = "unused_not_started/unused_not_started.rs"]
mod unused_not_started;
