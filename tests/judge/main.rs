//! Tests of the topic `judge`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "approved_passes/approved_passes.rs"]
mod approved_passes;

#[path = "approved_stop/approved_stop.rs"]
mod approved_stop;

#[path = "bad_verdict/bad_verdict.rs"]
mod bad_verdict;

#[path = "fixed_verdict/fixed_verdict.rs"]
mod fixed_verdict;

#[path = "judge_alone/judge_alone.rs"]
mod judge_alone;

#[path = "judge_blocks/judge_blocks.rs"]
mod judge_blocks;

#[path = "judge_main_flow/judge_main_flow.rs"]
mod judge_main_flow;

#[path = "judge_needs_next/judge_needs_next.rs"]
mod judge_needs_next;

#[path = "judge_only_id/judge_only_id.rs"]
mod judge_only_id;

#[path = "judge_placeholders/judge_placeholders.rs"]
mod judge_placeholders;

#[path = "judge_read_as_node/judge_read_as_node.rs"]
mod judge_read_as_node;

#[path = "on_error_values/on_error_values.rs"]
mod on_error_values;

#[path = "on_error_words/on_error_words.rs"]
mod on_error_words;

#[path = "plain_node_unaffected/plain_node_unaffected.rs"]
mod plain_node_unaffected;

#[path = "refuse_node/refuse_node.rs"]
mod refuse_node;

#[path = "refuse_stop/refuse_stop.rs"]
mod refuse_stop;

#[path = "run_node_refused/run_node_refused.rs"]
mod run_node_refused;
