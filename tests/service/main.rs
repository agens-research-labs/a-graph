//! Tests of the topic `service`: every test is a folder with its own files; the executable is run for real.

#[path = "../common/mod.rs"]
mod common;

#[path = "bad_request/bad_request.rs"]
mod bad_request;

#[path = "config_limits/config_limits.rs"]
mod config_limits;

#[path = "config_retention/config_retention.rs"]
mod config_retention;

#[path = "config_rules/config_rules.rs"]
mod config_rules;

#[path = "health/health.rs"]
mod health;

#[path = "host_ports/host_ports.rs"]
mod host_ports;

#[path = "host_refused/host_refused.rs"]
mod host_refused;

#[path = "invalid_yml/invalid_yml.rs"]
mod invalid_yml;

#[path = "no_programs/no_programs.rs"]
mod no_programs;

#[path = "paths_confined/paths_confined.rs"]
mod paths_confined;

#[path = "run_fails/run_fails.rs"]
mod run_fails;

#[path = "run_ok/run_ok.rs"]
mod run_ok;

#[path = "run_timeout/run_timeout.rs"]
mod run_timeout;

#[path = "runs_capped/runs_capped.rs"]
mod runs_capped;

#[path = "runs_capped_detached/runs_capped_detached.rs"]
mod runs_capped_detached;

#[path = "runs_events/runs_events.rs"]
mod runs_events;

#[path = "runs_fails/runs_fails.rs"]
mod runs_fails;

#[path = "runs_forgotten/runs_forgotten.rs"]
mod runs_forgotten;

#[path = "runs_level_off/runs_level_off.rs"]
mod runs_level_off;

#[path = "runs_level_steps/runs_level_steps.rs"]
mod runs_level_steps;

#[path = "runs_live/runs_live.rs"]
mod runs_live;

#[path = "runs_redacted/runs_redacted.rs"]
mod runs_redacted;

#[path = "runs_refused/runs_refused.rs"]
mod runs_refused;

#[path = "runs_replay/runs_replay.rs"]
mod runs_replay;

#[path = "runs_result/runs_result.rs"]
mod runs_result;

#[path = "runs_unknown/runs_unknown.rs"]
mod runs_unknown;

#[path = "sessions_apart/sessions_apart.rs"]
mod sessions_apart;

#[path = "together/together.rs"]
mod together;
