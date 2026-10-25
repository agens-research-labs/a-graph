//! Tests of the topic `events`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "content_redacted/content_redacted.rs"]
mod content_redacted;

#[path = "info_level_no_calls/info_level_no_calls.rs"]
mod info_level_no_calls;

#[path = "memory_read_logged/memory_read_logged.rs"]
mod memory_read_logged;

#[path = "run_events_logged/run_events_logged.rs"]
mod run_events_logged;

#[path = "tool_call_logged/tool_call_logged.rs"]
mod tool_call_logged;

#[path = "vectordb_read_logged/vectordb_read_logged.rs"]
mod vectordb_read_logged;

#[path = "tool_schema_ignored/tool_schema_ignored.rs"]
mod tool_schema_ignored;
