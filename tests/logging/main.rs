//! Tests of the topic `logging`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "batches_in_order/batches_in_order.rs"]
mod batches_in_order;

#[path = "build_follows_passport/build_follows_passport.rs"]
mod build_follows_passport;

#[path = "client_error_final/client_error_final.rs"]
mod client_error_final;

#[path = "error_event/error_event.rs"]
mod error_event;

#[path = "fixed_keys/fixed_keys.rs"]
mod fixed_keys;

#[path = "header_secrets_env/header_secrets_env.rs"]
mod header_secrets_env;

#[path = "last_attempt_dropped/last_attempt_dropped.rs"]
mod last_attempt_dropped;

#[path = "level_filter/level_filter.rs"]
mod level_filter;

#[path = "no_passport/no_passport.rs"]
mod no_passport;

#[path = "one_write_per_line/one_write_per_line.rs"]
mod one_write_per_line;

#[path = "redaction/redaction.rs"]
mod redaction;

#[path = "redirect_final/redirect_final.rs"]
mod redirect_final;

#[path = "server_error_retried/server_error_retried.rs"]
mod server_error_retried;

#[path = "silent_endpoint_timeout/silent_endpoint_timeout.rs"]
mod silent_endpoint_timeout;

#[path = "url_delivery/url_delivery.rs"]
mod url_delivery;

#[path = "warning_event/warning_event.rs"]
mod warning_event;

#[path = "error_detail_written/error_detail_written.rs"]
mod error_detail_written;
