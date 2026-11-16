//! Tests of the topic `resources`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "access_needs_url/access_needs_url.rs"]
mod access_needs_url;

#[path = "all_errors_together/all_errors_together.rs"]
mod all_errors_together;

#[path = "base64_not_valid/base64_not_valid.rs"]
mod base64_not_valid;

#[path = "builds_five/builds_five.rs"]
mod builds_five;

#[path = "mime_needs_base64/mime_needs_base64.rs"]
mod mime_needs_base64;

#[path = "missing_block/missing_block.rs"]
mod missing_block;

#[path = "secrets_keep_names/secrets_keep_names.rs"]
mod secrets_keep_names;

#[path = "source_two_origins/source_two_origins.rs"]
mod source_two_origins;
