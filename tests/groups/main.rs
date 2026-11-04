//! Tests of the topic `groups`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "concurrent_limit/concurrent_limit.rs"]
mod concurrent_limit;

#[path = "concurrent_needs_limit/concurrent_needs_limit.rs"]
mod concurrent_needs_limit;

#[path = "concurrent_one/concurrent_one.rs"]
mod concurrent_one;

#[path = "edge_ids_nodes_only/edge_ids_nodes_only.rs"]
mod edge_ids_nodes_only;

#[path = "edge_node_exists/edge_node_exists.rs"]
mod edge_node_exists;

#[path = "fan_not_empty/fan_not_empty.rs"]
mod fan_not_empty;

#[path = "group_order/group_order.rs"]
mod group_order;

#[path = "group_unknown_key/group_unknown_key.rs"]
mod group_unknown_key;

#[path = "group_with_out/group_with_out.rs"]
mod group_with_out;

#[path = "group_without_out/group_without_out.rs"]
mod group_without_out;

#[path = "limit_not_linear/limit_not_linear.rs"]
mod limit_not_linear;

#[path = "linear_each_input/linear_each_input.rs"]
mod linear_each_input;

#[path = "linear_in_order/linear_in_order.rs"]
mod linear_in_order;

#[path = "list_as_written/list_as_written.rs"]
mod list_as_written;

#[path = "loop_runs_twice/loop_runs_twice.rs"]
mod loop_runs_twice;

#[path = "mixed_output_refused/mixed_output_refused.rs"]
mod mixed_output_refused;

#[path = "nested_group_origin/nested_group_origin.rs"]
mod nested_group_origin;

#[path = "nested_groups_checked/nested_groups_checked.rs"]
mod nested_groups_checked;

#[path = "no_out_ok/no_out_ok.rs"]
mod no_out_ok;

#[path = "no_out_refused/no_out_refused.rs"]
mod no_out_refused;

#[path = "node_error_named/node_error_named.rs"]
mod node_error_named;

#[path = "node_twice/node_twice.rs"]
mod node_twice;

#[path = "one_node_graph/one_node_graph.rs"]
mod one_node_graph;

#[path = "one_start_one_end/one_start_one_end.rs"]
mod one_start_one_end;

#[path = "parallel_fails_fast/parallel_fails_fast.rs"]
mod parallel_fails_fast;

#[path = "parallel_packs/parallel_packs.rs"]
mod parallel_packs;

#[path = "parallel_threads/parallel_threads.rs"]
mod parallel_threads;

#[path = "parallel_vs_linear/parallel_vs_linear.rs"]
mod parallel_vs_linear;

#[path = "sequence_passes_on/sequence_passes_on.rs"]
mod sequence_passes_on;

#[path = "start_alone_needs_end/start_alone_needs_end.rs"]
mod start_alone_needs_end;

#[path = "start_first_end_last/start_first_end_last.rs"]
mod start_first_end_last;

#[path = "structured_packed/structured_packed.rs"]
mod structured_packed;
