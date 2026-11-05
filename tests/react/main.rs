//! Tests of the topic `react`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "controller_limits/controller_limits.rs"]
mod controller_limits;

#[path = "controller_memory/controller_memory.rs"]
mod controller_memory;

#[path = "controller_tokens/controller_tokens.rs"]
mod controller_tokens;

#[path = "duplicate_workers/duplicate_workers.rs"]
mod duplicate_workers;

#[path = "group_worker/group_worker.rs"]
mod group_worker;

#[path = "max_iterations/max_iterations.rs"]
mod max_iterations;

#[path = "nested_react/nested_react.rs"]
mod nested_react;

#[path = "no_answer/no_answer.rs"]
mod no_answer;

#[path = "one_worker/one_worker.rs"]
mod one_worker;

#[path = "react_kept/react_kept.rs"]
mod react_kept;

#[path = "rules_in_react/rules_in_react.rs"]
mod rules_in_react;

#[path = "several_workers/several_workers.rs"]
mod several_workers;

#[path = "unknown_worker/unknown_worker.rs"]
mod unknown_worker;
