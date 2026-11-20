//! Tests of the topic `prompts`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "declared_twice/declared_twice.rs"]
mod declared_twice;

#[path = "input_message/input_message.rs"]
mod input_message;

#[path = "message_required/message_required.rs"]
mod message_required;

#[path = "no_prompt_text/no_prompt_text.rs"]
mod no_prompt_text;

#[path = "no_prompt_two_messages/no_prompt_two_messages.rs"]
mod no_prompt_two_messages;

#[path = "placeholders_match/placeholders_match.rs"]
mod placeholders_match;

#[path = "prompt_base64_refused/prompt_base64_refused.rs"]
mod prompt_base64_refused;

#[path = "prompt_content_and_source/prompt_content_and_source.rs"]
mod prompt_content_and_source;

#[path = "prompt_file_missing/prompt_file_missing.rs"]
mod prompt_file_missing;

#[path = "prompt_file_no_content/prompt_file_no_content.rs"]
mod prompt_file_no_content;

#[path = "prompt_filled/prompt_filled.rs"]
mod prompt_filled;

#[path = "prompt_url_kept/prompt_url_kept.rs"]
mod prompt_url_kept;

#[path = "reserved_names/reserved_names.rs"]
mod reserved_names;

#[path = "url_prompt_read/url_prompt_read.rs"]
mod url_prompt_read;

#[path = "url_prompt_refused/url_prompt_refused.rs"]
mod url_prompt_refused;
