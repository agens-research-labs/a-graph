//! The Loader: every test is a folder with its own yml files.


pub use a_graph::loader::load;
pub use a_graph::schema::key;
pub use super::{yml_in};

/// Loads `<dir>/<name>.yml` (a file of the test whose folder is `dir`) and returns the (rule, path) of every error.
pub fn errors(dir: &std::path::Path, name: &str) -> Vec<(String, String)> {
    load(&yml_in(dir, name)).unwrap_err().0.iter().map(|e| (e.rule.code().to_string(), e.path.to_string())).collect()
}

/// Whether the file reports `rule` at `path`.
pub fn has(dir: &std::path::Path, name: &str, rule: &str, path: &str) -> bool {
    errors(dir, name).iter().any(|(r, p)| r == rule && p == path)
}

pub fn loads(dir: &std::path::Path, name: &str) -> bool {
    load(&yml_in(dir, name)).is_ok()
}




































