//! Tests of the topic `verbose`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "colors_on_terminal/colors_on_terminal.rs"]
mod colors_on_terminal;

#[path = "content_hidden/content_hidden.rs"]
mod content_hidden;

#[path = "detail_shows_calls/detail_shows_calls.rs"]
mod detail_shows_calls;

#[path = "error_shown_red/error_shown_red.rs"]
mod error_shown_red;

#[path = "groups_indented/groups_indented.rs"]
mod groups_indented;

#[path = "off_shows_nothing/off_shows_nothing.rs"]
mod off_shows_nothing;

#[path = "reasoning_in_gray/reasoning_in_gray.rs"]
mod reasoning_in_gray;

#[path = "steps_shows_nodes/steps_shows_nodes.rs"]
mod steps_shows_nodes;

#[path = "tool_call_shown/tool_call_shown.rs"]
mod tool_call_shown;

#[path = "error_detail/error_detail.rs"]
mod error_detail;

#[path = "schema_attempt_shown/schema_attempt_shown.rs"]
mod schema_attempt_shown;
