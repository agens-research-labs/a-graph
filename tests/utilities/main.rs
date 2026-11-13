//! Tests of the topic `utilities`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "board_gets_changed/board_gets_changed.rs"]
mod board_gets_changed;

#[path = "board_write_utility/board_write_utility.rs"]
mod board_write_utility;

#[path = "builds/builds.rs"]
mod builds;

#[path = "call_timeout/call_timeout.rs"]
mod call_timeout;

#[path = "cannot_start/cannot_start.rs"]
mod cannot_start;

#[path = "exit_error/exit_error.rs"]
mod exit_error;

#[path = "failing_stops/failing_stops.rs"]
mod failing_stops;

#[path = "file_has_no_source/file_has_no_source.rs"]
mod file_has_no_source;

#[path = "file_in_text_place/file_in_text_place.rs"]
mod file_in_text_place;

#[path = "file_input/file_input.rs"]
mod file_input;

#[path = "file_output/file_output.rs"]
mod file_output;

#[path = "file_to_model_refused/file_to_model_refused.rs"]
mod file_to_model_refused;

#[path = "from_message_needs_file/from_message_needs_file.rs"]
mod from_message_needs_file;

#[path = "from_message_needs_utility/from_message_needs_utility.rs"]
mod from_message_needs_utility;

#[path = "header_env_missing/header_env_missing.rs"]
mod header_env_missing;

#[path = "host_refused/host_refused.rs"]
mod host_refused;

#[path = "http_error/http_error.rs"]
mod http_error;

#[path = "http_roundtrip/http_roundtrip.rs"]
mod http_roundtrip;

#[path = "http_rules/http_rules.rs"]
mod http_rules;

#[path = "input_chain/input_chain.rs"]
mod input_chain;

#[path = "live_pdf_to_md/live_pdf_to_md.rs"]
mod live_pdf_to_md;

#[path = "media_file_utility/media_file_utility.rs"]
mod media_file_utility;

#[path = "media_from_message/media_from_message.rs"]
mod media_from_message;

#[path = "media_node_utility/media_node_utility.rs"]
mod media_node_utility;

#[path = "media_not_from_node/media_not_from_node.rs"]
mod media_not_from_node;

#[path = "memory_gets_changed/memory_gets_changed.rs"]
mod memory_gets_changed;

#[path = "memory_read_utility/memory_read_utility.rs"]
mod memory_read_utility;

#[path = "memory_write_utility/memory_write_utility.rs"]
mod memory_write_utility;

#[path = "mime_from_extension/mime_from_extension.rs"]
mod mime_from_extension;

#[path = "name_unique/name_unique.rs"]
mod name_unique;

#[path = "not_json/not_json.rs"]
mod not_json;

#[path = "output_once/output_once.rs"]
mod output_once;

#[path = "output_to_next/output_to_next.rs"]
mod output_to_next;

#[path = "single_id/single_id.rs"]
mod single_id;

#[path = "stdio_rules/stdio_rules.rs"]
mod stdio_rules;

#[path = "text_roundtrip/text_roundtrip.rs"]
mod text_roundtrip;

#[path = "timeout_missing/timeout_missing.rs"]
mod timeout_missing;

#[path = "timeout_zero/timeout_zero.rs"]
mod timeout_zero;

#[path = "unknown_in_message/unknown_in_message.rs"]
mod unknown_in_message;

#[path = "unknown_in_places/unknown_in_places.rs"]
mod unknown_in_places;

#[path = "vectordb_utility/vectordb_utility.rs"]
mod vectordb_utility;

#[path = "wrong_shape/wrong_shape.rs"]
mod wrong_shape;

#[path = "stderr_kept_out/stderr_kept_out.rs"]
mod stderr_kept_out;
