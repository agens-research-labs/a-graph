//! The utilities: every test is a folder with its own yml and, when a program is needed, its own scripts.

pub use a_graph::loader::load;
pub use a_graph::schema::key;
pub use a_graph::utilities::*;
pub use serde_json::{Value, json};
pub use super::http::{Resp, listen};
pub use super::{own_yml, yml_in, yml_with};

/// The manager built from the `utilities` block of a yml.
pub fn manager(src: &str) -> UtilityManager {
    let mut doc = load(src).unwrap_or_else(|e| panic!("the Loader rejected the yml: {e}"));
    UtilityManager::build(doc.take(key::UTILITIES)).unwrap_or_else(|e| panic!("{e}"))
}

/// The (rule, path) of every error the manager reports.
pub fn build_errors(src: &str) -> Vec<(String, String)> {
    let mut doc = load(src).unwrap_or_else(|e| panic!("the Loader rejected the yml: {e}"));
    UtilityManager::build(doc.take(key::UTILITIES)).unwrap_err().0.iter().map(|e| (e.rule.code().to_string(), e.path.to_string())).collect()
}

/// The (rule, path) of every error the Loader reports.
pub fn load_errors(src: &str) -> Vec<(String, String)> {
    load(src).unwrap_err().0.iter().map(|e| (e.rule.code().to_string(), e.path.to_string())).collect()
}
