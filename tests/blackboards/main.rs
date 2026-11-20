//! Tests of the topic `blackboards`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "board_not_read_refused/board_not_read_refused.rs"]
mod board_not_read_refused;

#[path = "board_read_fills/board_read_fills.rs"]
mod board_read_fills;

#[path = "boards_valid/boards_valid.rs"]
mod boards_valid;

#[path = "cleanup_write_refused/cleanup_write_refused.rs"]
mod cleanup_write_refused;

#[path = "cleanup_keeps_header/cleanup_keeps_header.rs"]
mod cleanup_keeps_header;

#[path = "cleanup_memory/cleanup_memory.rs"]
mod cleanup_memory;

#[path = "file_header_first/file_header_first.rs"]
mod file_header_first;

#[path = "file_missing/file_missing.rs"]
mod file_missing;

#[path = "file_no_header/file_no_header.rs"]
mod file_no_header;

#[path = "header_rules/header_rules.rs"]
mod header_rules;

#[path = "leftover_tmp_ignored/leftover_tmp_ignored.rs"]
mod leftover_tmp_ignored;

#[path = "link_refused/link_refused.rs"]
mod link_refused;

#[path = "memory_board_writes/memory_board_writes.rs"]
mod memory_board_writes;

#[path = "node_board_cycle/node_board_cycle.rs"]
mod node_board_cycle;

#[path = "node_cleanup/node_cleanup.rs"]
mod node_cleanup;

#[path = "parallel_writes/parallel_writes.rs"]
mod parallel_writes;

#[path = "programs_share_file/programs_share_file.rs"]
mod programs_share_file;

#[path = "unknown_board/unknown_board.rs"]
mod unknown_board;

#[path = "url_board_refused/url_board_refused.rs"]
mod url_board_refused;

#[path = "write_no_header/write_no_header.rs"]
mod write_no_header;
