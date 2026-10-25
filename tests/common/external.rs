//! The external managers: every test is a folder with its own yml files.


pub use a_graph::external::{External, MemoryBackend, VectorBackend};
pub use a_graph::loader::load;
pub use a_graph::schema::key;
pub use super::{yml_in};

pub fn build_text(src: &str) -> Result<External, Vec<(String, String)>> {
    let mut doc = load(src).unwrap_or_else(|e| panic!("the Loader rejected the yml: {e}"));
    External::build(doc.take(key::EXTERNAL)).map_err(|e| e.0.iter().map(|e| (e.rule.code().to_string(), e.path.to_string())).collect())
}

/// The external managers of `<dir>/<name>.yml`, a file of the test whose folder is `dir`.
pub fn build(dir: &std::path::Path, name: &str) -> Result<External, Vec<(String, String)>> {
    build_text(&yml_in(dir, name))
}

pub fn has(dir: &std::path::Path, name: &str, rule: &str, path: &str) -> bool {
    build(dir, name).unwrap_err().iter().any(|(r, p)| r == rule && p == path)
}









