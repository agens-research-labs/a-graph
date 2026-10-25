//! Tests of the topic `external`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "all_errors_together/all_errors_together.rs"]
mod all_errors_together;

#[path = "builds_three/builds_three.rs"]
mod builds_three;

#[path = "missing_block/missing_block.rs"]
mod missing_block;

#[path = "search_keeps_values/search_keeps_values.rs"]
mod search_keeps_values;
