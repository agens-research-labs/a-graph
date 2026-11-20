//! Tests of the topic `resources`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "all_errors_together/all_errors_together.rs"]
mod all_errors_together;

#[path = "base64_not_valid/base64_not_valid.rs"]
mod base64_not_valid;

#[path = "builds_five/builds_five.rs"]
mod builds_five;

#[path = "headers_need_url/headers_need_url.rs"]
mod headers_need_url;

#[path = "mime_not_with_path/mime_not_with_path.rs"]
mod mime_not_with_path;

#[path = "missing_block/missing_block.rs"]
mod missing_block;

#[path = "secrets_keep_names/secrets_keep_names.rs"]
mod secrets_keep_names;

#[path = "source_two_origins/source_two_origins.rs"]
mod source_two_origins;
