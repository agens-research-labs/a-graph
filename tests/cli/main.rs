//! Tests of the topic `cli`: every test is a folder with its own files; the executable is run for real.

#[path = "../common/mod.rs"]
mod common;

#[path = "ctrl_c_stops/ctrl_c_stops.rs"]
mod ctrl_c_stops;

#[path = "check_invalid/check_invalid.rs"]
mod check_invalid;

#[path = "check_valid/check_valid.rs"]
mod check_valid;

#[path = "invalid_not_run/invalid_not_run.rs"]
mod invalid_not_run;

#[path = "log_on_stderr/log_on_stderr.rs"]
mod log_on_stderr;

#[path = "log_on_stdout/log_on_stdout.rs"]
mod log_on_stdout;

#[path = "print_outputs/print_outputs.rs"]
mod print_outputs;

#[path = "run_chat_mode/run_chat_mode.rs"]
mod run_chat_mode;

#[path = "run_fails/run_fails.rs"]
mod run_fails;

#[path = "run_json/run_json.rs"]
mod run_json;

#[path = "run_stdin/run_stdin.rs"]
mod run_stdin;

#[path = "usage_errors/usage_errors.rs"]
mod usage_errors;
