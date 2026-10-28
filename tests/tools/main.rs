//! Tests of the topic `tools`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "count_restarts/count_restarts.rs"]
mod count_restarts;

#[path = "http_failure/http_failure.rs"]
mod http_failure;

#[path = "http_post_header/http_post_header.rs"]
mod http_post_header;

#[path = "http_redirect/http_redirect.rs"]
mod http_redirect;

#[path = "http_timeout/http_timeout.rs"]
mod http_timeout;

#[path = "http_tool_rules/http_tool_rules.rs"]
mod http_tool_rules;

#[path = "node_tools_by_id/node_tools_by_id.rs"]
mod node_tools_by_id;

#[path = "stdio_cannot_start/stdio_cannot_start.rs"]
mod stdio_cannot_start;

#[path = "stdio_exit_error/stdio_exit_error.rs"]
mod stdio_exit_error;

#[path = "stdio_not_json/stdio_not_json.rs"]
mod stdio_not_json;

#[path = "stdio_roundtrip/stdio_roundtrip.rs"]
mod stdio_roundtrip;

#[path = "stdio_timeout/stdio_timeout.rs"]
mod stdio_timeout;

#[path = "stdio_tool_rules/stdio_tool_rules.rs"]
mod stdio_tool_rules;

#[path = "tool_bad_arguments/tool_bad_arguments.rs"]
mod tool_bad_arguments;

#[path = "tool_headers/tool_headers.rs"]
mod tool_headers;

#[path = "tool_max_iter/tool_max_iter.rs"]
mod tool_max_iter;

#[path = "tool_over_max_iter/tool_over_max_iter.rs"]
mod tool_over_max_iter;

#[path = "tool_params_object/tool_params_object.rs"]
mod tool_params_object;

#[path = "tool_params_not_json/tool_params_not_json.rs"]
mod tool_params_not_json;

#[path = "tool_params_schema/tool_params_schema.rs"]
mod tool_params_schema;

#[path = "tool_rounds_limit/tool_rounds_limit.rs"]
mod tool_rounds_limit;

#[path = "two_calls_in_order/two_calls_in_order.rs"]
mod two_calls_in_order;

#[path = "unknown_tool/unknown_tool.rs"]
mod unknown_tool;
