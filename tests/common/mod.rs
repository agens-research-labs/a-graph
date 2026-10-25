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

/// The text of `<dir>/<stem>.yml`: a graph of the test whose folder is `dir`.
pub fn yml_in(dir: &std::path::Path, stem: &str) -> String {
    trace_token();
    let path = dir.join(format!("{stem}.yml"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
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
