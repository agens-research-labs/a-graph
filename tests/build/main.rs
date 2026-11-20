//! Tests of the topic `build`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "all_errors_logged/all_errors_logged.rs"]
mod all_errors_logged;

#[path = "check_before_call/check_before_call.rs"]
mod check_before_call;

#[path = "env_missing/env_missing.rs"]
mod env_missing;

#[path = "host_refused/host_refused.rs"]
mod host_refused;

#[path = "loader_errors_alone/loader_errors_alone.rs"]
mod loader_errors_alone;

#[path = "max_tokens_anthropic/max_tokens_anthropic.rs"]
mod max_tokens_anthropic;

#[path = "model_lacks_feature/model_lacks_feature.rs"]
mod model_lacks_feature;

#[path = "node_needs_model/node_needs_model.rs"]
mod node_needs_model;

#[path = "url_header_env/url_header_env.rs"]
mod url_header_env;

#[path = "valid_builds/valid_builds.rs"]
mod valid_builds;

#[path = "warnings_logged/warnings_logged.rs"]
mod warnings_logged;
