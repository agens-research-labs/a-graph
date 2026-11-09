//! Tests of the topic `service`: every test is a folder with its own files; the executable is run for real.

#[path = "../common/mod.rs"]
mod common;

#[path = "ambient_config/ambient_config.rs"]
mod ambient_config;

#[path = "ambient_refused/ambient_refused.rs"]
mod ambient_refused;

#[path = "bad_request/bad_request.rs"]
mod bad_request;

#[path = "body_too_large/body_too_large.rs"]
mod body_too_large;

#[path = "busy_costs_nothing/busy_costs_nothing.rs"]
mod busy_costs_nothing;

#[path = "config_limits/config_limits.rs"]
mod config_limits;

#[path = "config_retention/config_retention.rs"]
mod config_retention;

#[path = "config_storage/config_storage.rs"]
mod config_storage;

#[path = "config_rules/config_rules.rs"]
mod config_rules;

#[path = "env_allowed/env_allowed.rs"]
mod env_allowed;

#[path = "env_config/env_config.rs"]
mod env_config;

#[path = "env_log_refused/env_log_refused.rs"]
mod env_log_refused;

#[path = "env_not_listed/env_not_listed.rs"]
mod env_not_listed;

#[path = "env_wrong_host/env_wrong_host.rs"]
mod env_wrong_host;

#[path = "health/health.rs"]
mod health;

#[path = "host_ports/host_ports.rs"]
mod host_ports;

#[path = "host_refused/host_refused.rs"]
mod host_refused;

#[path = "invalid_yml/invalid_yml.rs"]
mod invalid_yml;

#[path = "log_host_refused/log_host_refused.rs"]
mod log_host_refused;

#[path = "no_aliases/no_aliases.rs"]
mod no_aliases;

#[path = "no_tags/no_tags.rs"]
mod no_tags;

#[path = "no_programs/no_programs.rs"]
mod no_programs;

#[path = "paths_confined/paths_confined.rs"]
mod paths_confined;

#[path = "run_fails/run_fails.rs"]
mod run_fails;

#[path = "run_ok/run_ok.rs"]
mod run_ok;

#[path = "read_timeout/read_timeout.rs"]
mod read_timeout;

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

#[path = "session_case/session_case.rs"]
mod session_case;

#[path = "session_foreign_kept/session_foreign_kept.rs"]
mod session_foreign_kept;

#[path = "session_full/session_full.rs"]
mod session_full;

#[path = "session_in_use_kept/session_in_use_kept.rs"]
mod session_in_use_kept;

#[path = "session_leftover/session_leftover.rs"]
mod session_leftover;

#[path = "session_mark_refused/session_mark_refused.rs"]
mod session_mark_refused;

#[path = "session_swept/session_swept.rs"]
mod session_swept;

#[path = "sessions_apart/sessions_apart.rs"]
mod sessions_apart;

#[path = "shutdown_bounded/shutdown_bounded.rs"]
mod shutdown_bounded;

#[path = "shutdown_stops/shutdown_stops.rs"]
mod shutdown_stops;

#[path = "shutdown_waits/shutdown_waits.rs"]
mod shutdown_waits;

#[path = "together/together.rs"]
mod together;

#[path = "too_many_events/too_many_events.rs"]
mod too_many_events;

#[path = "error_kept_out/error_kept_out.rs"]
mod error_kept_out;
