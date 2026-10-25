//! Tests of the topic `trace`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "collector_down/collector_down.rs"]
mod collector_down;

#[path = "content_redacted/content_redacted.rs"]
mod content_redacted;

#[path = "header_env_missing/header_env_missing.rs"]
mod header_env_missing;

#[path = "no_trace_block/no_trace_block.rs"]
mod no_trace_block;

#[path = "parallel_overlap/parallel_overlap.rs"]
mod parallel_overlap;

#[path = "parent_links/parent_links.rs"]
mod parent_links;

#[path = "run_error_failed/run_error_failed.rs"]
mod run_error_failed;

#[path = "sends_otlp_json/sends_otlp_json.rs"]
mod sends_otlp_json;

#[path = "times_nest/times_nest.rs"]
mod times_nest;
