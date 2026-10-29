//! The embedding managers: every test is a folder with its own yml files.

pub use a_graph::embeddings::EmbeddingManager;
pub use a_graph::loader::load;
pub use a_graph::models::ModelError;
pub use a_graph::schema::key;
pub use super::{own_yml, yml_in, yml_with};

/// The embedding manager built from the `embeddings` block of a yml.
pub fn manager(src: &str) -> EmbeddingManager {
    let mut doc = load(src).unwrap_or_else(|e| panic!("the Loader rejected the yml: {e}"));
    EmbeddingManager::build(doc.take(key::EMBEDDINGS)).unwrap_or_else(|e| panic!("{e}"))
}

/// The (rule, path) of every error the Loader finds.
pub fn load_errors(src: &str) -> Vec<(String, String)> {
    load(src).unwrap_err().0.iter().map(|e| (e.rule.code().to_string(), e.path.to_string())).collect()
}
