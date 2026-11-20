//! Tests of the topic `memory`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "adds_to_record/adds_to_record.rs"]
mod adds_to_record;

#[path = "chat_required/chat_required.rs"]
mod chat_required;

#[path = "chat_unknown_embedding/chat_unknown_embedding.rs"]
mod chat_unknown_embedding;

#[path = "concurrent_writes/concurrent_writes.rs"]
mod concurrent_writes;

#[path = "embedding_error/embedding_error.rs"]
mod embedding_error;

#[path = "embedding_similar/embedding_similar.rs"]
mod embedding_similar;

#[path = "file_mode_kept/file_mode_kept.rs"]
mod file_mode_kept;

#[path = "json_file_builds/json_file_builds.rs"]
mod json_file_builds;

#[path = "length_zero/length_zero.rs"]
mod length_zero;

#[path = "link_refused/link_refused.rs"]
mod link_refused;

#[path = "live_chat_history/live_chat_history.rs"]
mod live_chat_history;

#[path = "live_similar_exchange/live_similar_exchange.rs"]
mod live_similar_exchange;

#[path = "live_summary_loop/live_summary_loop.rs"]
mod live_summary_loop;

#[path = "makes_summary/makes_summary.rs"]
mod makes_summary;

#[path = "memory_file_kept_out/memory_file_kept_out.rs"]
mod memory_file_kept_out;

#[path = "memory_file_rules/memory_file_rules.rs"]
mod memory_file_rules;

#[path = "memory_ref_linked/memory_ref_linked.rs"]
mod memory_ref_linked;

#[path = "memory_ref_rules/memory_ref_rules.rs"]
mod memory_ref_rules;

#[path = "no_file_is_empty/no_file_is_empty.rs"]
mod no_file_is_empty;

#[path = "no_input_saves_answer/no_input_saves_answer.rs"]
mod no_input_saves_answer;

#[path = "no_synt_no_placeholder/no_synt_no_placeholder.rs"]
mod no_synt_no_placeholder;

#[path = "other_node_record/other_node_record.rs"]
mod other_node_record;

#[path = "read_roles/read_roles.rs"]
mod read_roles;

#[path = "reads_history/reads_history.rs"]
mod reads_history;

#[path = "reasoning_not_saved/reasoning_not_saved.rs"]
mod reasoning_not_saved;

#[path = "saves_node_message/saves_node_message.rs"]
mod saves_node_message;

#[path = "saves_vector/saves_vector.rs"]
mod saves_vector;

#[path = "similar_by_node_message/similar_by_node_message.rs"]
mod similar_by_node_message;

#[path = "summaries_overlap/summaries_overlap.rs"]
mod summaries_overlap;

#[path = "summary_error/summary_error.rs"]
mod summary_error;

#[path = "summary_own_prompt/summary_own_prompt.rs"]
mod summary_own_prompt;

#[path = "summary_url_prompt/summary_url_prompt.rs"]
mod summary_url_prompt;

#[path = "synt_in_prompt/synt_in_prompt.rs"]
mod synt_in_prompt;

#[path = "synt_placeholder_unused/synt_placeholder_unused.rs"]
mod synt_placeholder_unused;

#[path = "synt_prompt_needs_both/synt_prompt_needs_both.rs"]
mod synt_prompt_needs_both;

#[path = "synt_unknown_model/synt_unknown_model.rs"]
mod synt_unknown_model;

#[path = "vectordb_refuses_synt/vectordb_refuses_synt.rs"]
mod vectordb_refuses_synt;

#[path = "write_roles/write_roles.rs"]
mod write_roles;

#[path = "writes_exchange/writes_exchange.rs"]
mod writes_exchange;
