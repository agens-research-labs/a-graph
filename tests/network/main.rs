//! Tests of the topic `network`: every test is a folder with its own files.

#[path = "../common/mod.rs"]
mod common;

#[path = "batch_hosts/batch_hosts.rs"]
mod batch_hosts;

#[path = "destinations_listed/destinations_listed.rs"]
mod destinations_listed;

#[path = "file_not_network/file_not_network.rs"]
mod file_not_network;

#[path = "host_ports/host_ports.rs"]
mod host_ports;

#[path = "hosts_lowercase/hosts_lowercase.rs"]
mod hosts_lowercase;

#[path = "hosts_before_env/hosts_before_env.rs"]
mod hosts_before_env;

#[path = "logging_hosts/logging_hosts.rs"]
mod logging_hosts;

#[path = "model_hosts/model_hosts.rs"]
mod model_hosts;

#[path = "no_permissions/no_permissions.rs"]
mod no_permissions;

#[path = "refused_log_url/refused_log_url.rs"]
mod refused_log_url;

#[path = "resource_hosts/resource_hosts.rs"]
mod resource_hosts;

#[path = "trace_before_env/trace_before_env.rs"]
mod trace_before_env;

#[path = "url_no_host/url_no_host.rs"]
mod url_no_host;

#[path = "url_parsed/url_parsed.rs"]
mod url_parsed;

#[path = "valid_hosts_ok/valid_hosts_ok.rs"]
mod valid_hosts_ok;
