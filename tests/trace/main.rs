//! Tests of the topic `trace`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "cancelled_model/cancelled_model.rs"]
mod cancelled_model;

#[path = "collector_down/collector_down.rs"]
mod collector_down;

#[path = "content_policy/content_policy.rs"]
mod content_policy;

#[path = "content_redacted/content_redacted.rs"]
mod content_redacted;

#[path = "correlated_logs/correlated_logs.rs"]
mod correlated_logs;

#[path = "error_detail_hidden/error_detail_hidden.rs"]
mod error_detail_hidden;

#[path = "error_detail_shown/error_detail_shown.rs"]
mod error_detail_shown;

#[path = "header_env_missing/header_env_missing.rs"]
mod header_env_missing;

#[path = "no_trace_block/no_trace_block.rs"]
mod no_trace_block;

#[path = "outbound_metadata/outbound_metadata.rs"]
mod outbound_metadata;

#[path = "outbound_parallel/outbound_parallel.rs"]
mod outbound_parallel;

#[path = "parallel_overlap/parallel_overlap.rs"]
mod parallel_overlap;

#[path = "parent_links/parent_links.rs"]
mod parent_links;

#[path = "partial_success/partial_success.rs"]
mod partial_success;

#[path = "run_error_failed/run_error_failed.rs"]
mod run_error_failed;

#[path = "sends_otlp_json/sends_otlp_json.rs"]
mod sends_otlp_json;

#[path = "times_nest/times_nest.rs"]
mod times_nest;

#[path = "schema_attempt_traced/schema_attempt_traced.rs"]
mod schema_attempt_traced;

#[path = "url_read_traced/url_read_traced.rs"]
mod url_read_traced;
