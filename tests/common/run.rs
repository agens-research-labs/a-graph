//! Building the program and running a node. Every test is a folder with its own yml files. The logs the tests read are written to
//! `target/test-output/`.

pub use super::{yml_in, yml_with};
pub use a_graph::run::build;

pub fn set_env() {
    // SAFETY: only sets the values the yml files name; they are never used to call anything.
    unsafe {
        std::env::set_var("ANTHROPIC_API_KEY", "a");
        std::env::set_var("OPENAI_API_KEY", "o");
    }
}

/// The text of `<dir>/<name>.yml`, a file of the test whose folder is `dir`.
pub fn yml(dir: &std::path::Path, name: &str) -> String {
    yml_in(dir, name)
}

/// The lines of the log `target/test-output/<name>.log`.
pub fn log_lines(name: &str) -> Vec<serde_json::Value> {
    std::fs::read_to_string(format!("target/test-output/{name}.log"))
        .unwrap_or_default()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

/// The (rule, path) pairs `build` reports for the file.
pub fn errors(dir: &std::path::Path, name: &str) -> Vec<(String, String)> {
    set_env();
    build(&yml(dir, name))
        .err()
        .unwrap()
        .0
        .iter()
        .map(|e| (e.rule.code().to_string(), e.path.to_string()))
        .collect()
}

pub fn has(dir: &std::path::Path, name: &str, rule: &str, path: &str) -> bool {
    errors(dir, name).iter().any(|(r, p)| r == rule && p == path)
}
