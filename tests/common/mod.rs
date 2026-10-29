//! Code shared by the integration tests (never data: every test has its own folder with all the files
//! it reads). Tests are run from the root of the project, so the paths are relative to it.

#![allow(dead_code, unused_imports)]

pub mod http;
pub mod blackboards;
pub mod cli;
pub mod embeddings;
pub mod external;
pub mod graph;
pub mod graph_edges;
pub mod loader;
pub mod logging;
pub mod logging_delivery;
pub mod logging_logger;
pub mod network;
pub mod resources;
pub mod resources_skills;
pub mod run;
pub mod run_batch;
pub mod run_graph;
pub mod run_judge;
pub mod run_live;
pub mod run_mcp;
pub mod run_node;
pub mod run_react;
pub mod run_skills;
pub mod run_tools;
pub mod service;
pub mod toolmock;
pub mod utilities;
pub mod trace;
pub mod verbose;

/// The fixtures with a `trace` block name the variable `TRACE_TOKEN` for a header, and the tracer reads it
/// when the program is built: it is given a value, once, whenever a test reads a yml.
fn trace_token() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    // SAFETY: the value is never used to reach a real collector.
    ONCE.call_once(|| unsafe { std::env::set_var("TRACE_TOKEN", "t") });
}

/// The text of `<dir>/<stem>.yml`: a graph of the test whose folder is `dir`. Its mock servers are
/// named `{{port:<name>}}` (see `with_ports`); none is given here, the test only builds the graph,
/// so every name becomes 1 (nothing listens there, a call would be refused).
pub fn yml_in(dir: &std::path::Path, stem: &str) -> String {
    yml_with(dir, stem, &[])
}

/// Like `yml_in`, with the ports of the mock servers the test started. A `{{port:<name>}}` whose
/// name the test did not pass is a bug (a typo in the fixture, or a server the test forgot to start):
/// it panics, so that a test does not pass on "connection refused" to port 1 by mistake. A test that
/// only builds the graph (no servers, so no ports) is handled by `yml_in`.
pub fn yml_with(dir: &std::path::Path, stem: &str, ports: &[(&str, u16)]) -> String {
    trace_token();
    let path = dir.join(format!("{stem}.yml"));
    with_ports(&std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display())), ports)
}

/// The fixtures never name a fixed port: a mock server listens on a free one, so that tests (and two
/// `cargo test` at once) never meet on a port. Every `{{port:<name>}}` becomes the port of that name;
/// when the caller passed some ports, a name the caller did not pass panics (a bug: a typo in the
/// fixture, or a server the test forgot to start). When the caller passed no ports, the test only
/// builds the graph and never calls a server: every name becomes 1 (nothing listens there).
pub fn with_ports(text: &str, ports: &[(&str, u16)]) -> String {
    if ports.is_empty() {
        return replace_ports(text, |_| 1);
    }
    replace_ports(text, |name| {
        ports.iter().find(|(n, _)| *n == name).unwrap_or_else(|| panic!("fixture names port `{name}` but the test did not pass it")).1
    })
}

fn replace_ports(text: &str, mut port_of: impl FnMut(&str) -> u16) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("{{port:") {
        out.push_str(&rest[..at]);
        let after = &rest[at + "{{port:".len()..];
        let end = after.find("}}").unwrap_or_else(|| panic!("a `{{{{port:` that does not end: {}", &rest[at..]));
        let name = &after[..end];
        out.push_str(&port_of(name).to_string());
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    out
}

/// Writes `text` (a yml with its ports) under `target/test-ports/`, for a program that reads a file:
/// the name holds the process, so two `cargo test` at once write two files.
pub fn ported_file(stem: &str, text: &str) -> String {
    let path = format!("target/test-ports/{stem}-{}.yml", std::process::id());
    std::fs::create_dir_all("target/test-ports").unwrap();
    std::fs::write(&path, text).unwrap();
    path
}

/// The graph named like the folder of the test: `<dir>/<name of dir>.yml`.
pub fn own_yml(dir: &std::path::Path) -> String {
    yml_in(dir, &dir.file_name().unwrap().to_string_lossy())
}

/// Copies `<dir>/<template>` to `<target>` (a path under `target/`, relative to the project), so a
/// test can change the copy and the file of the test stays as it is.
pub fn fresh_copy(dir: &std::path::Path, template: &str, target: &str) {
    let to = std::path::Path::new(target);
    std::fs::create_dir_all(to.parent().unwrap()).unwrap();
    let from = dir.join(template);
    std::fs::copy(&from, to).unwrap_or_else(|e| panic!("cannot copy {}: {e}", from.display()));
}

/// Prepares the file a test writes to (a path under `target/`, relative to the project): its folder
/// exists and the file is empty.
pub fn clean_output(target: &str) {
    let path = std::path::Path::new(target);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let _ = std::fs::remove_file(path);
}

/// Builds the MCP server of the tests (the project `tests/mcp_server`) once, into `target/mcp-test-server`.
/// It is a project of its own, so that building it leaves the build of the crate under test alone.
pub fn ensure_mcp_server() {
    static BUILT: std::sync::Once = std::sync::Once::new();
    BUILT.call_once(|| {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
        let status = std::process::Command::new(cargo).args(["build", "--quiet", "--manifest-path", "tests/mcp_server/Cargo.toml", "--target-dir", "target/mcp-test-server"]).status().expect("cannot run cargo");
        assert!(status.success(), "the MCP server of the tests did not build");
    });
}
