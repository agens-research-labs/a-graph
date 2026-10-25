//! The resource managers: every test is a folder with its own yml files
//! (and, for the skills, its own skill folders under `files/`).


pub use a_graph::loader::load;
pub use a_graph::resources::{GeminiImageSize, MediaKind, MediaProvider, McpTransport, Origin, PromptBody, Resources, Source, ToolTransport};
pub use a_graph::schema::key;
pub use super::{yml_in};

pub fn build_text(src: &str) -> Result<Resources, a_graph::Errors> {
    let mut doc = load(src).unwrap_or_else(|e| panic!("the Loader rejected the yml: {e}"));
    Resources::build(doc.take(key::RESOURCES))
}

/// The resources of `<dir>/<name>.yml`, a file of the test whose folder is `dir`.
pub fn build(dir: &std::path::Path, name: &str) -> Result<Resources, a_graph::Errors> {
    build_text(&yml_in(dir, name))
}

/// The (rule, path) of every error a manager reports for the file.
pub fn errors(dir: &std::path::Path, name: &str) -> Vec<(String, String)> {
    build(dir, name).unwrap_err().0.iter().map(|e| (e.rule.code().to_string(), e.path.to_string())).collect()
}

pub fn has(dir: &std::path::Path, name: &str, rule: &str, path: &str) -> bool {
    errors(dir, name).iter().any(|(r, p)| r == rule && p == path)
}



























