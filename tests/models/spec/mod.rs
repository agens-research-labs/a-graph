//! The `models` block read from the yml: the rules about the content of a model. Every case is a
//! folder with its own yml files.

use a_graph::loader::load;
use a_graph::models::ModelManager;
use a_graph::schema::key;
use super::common::{yml_in};

/// The (rule, path) pairs the managers report for `<dir>/<name>.yml`, a file of the test whose folder is `dir`. The Loader must accept it.
fn errors(dir: &std::path::Path, name: &str) -> Vec<(String, String)> {
    set_env();
    let mut doc = load(&yml_in(dir, name)).unwrap_or_else(|e| panic!("the Loader rejected the yml: {e}"));
    match ModelManager::build(doc.take(key::MODELS)) {
        Ok(_) => vec![],
        Err(e) => e.0.iter().map(|e| (e.rule.code().to_string(), e.path.to_string())).collect(),
    }
}

fn has(dir: &std::path::Path, name: &str, rule: &str, path: &str) -> bool {
    errors(dir, name).iter().any(|(r, p)| r == rule && p == path)
}

/// The environment variables the yml files name.
fn set_env() {
    // SAFETY: the values are never used to call anything.
    unsafe {
        std::env::set_var("ANTHROPIC_API_KEY", "a");
        std::env::set_var("OPENAI_API_KEY", "o");
    }
}

#[path = "bedrock_text_only/bedrock_text_only.rs"]
mod bedrock_text_only;

#[path = "no_batch_anthropic/no_batch_anthropic.rs"]
mod no_batch_anthropic;

#[path = "workspace_anthropic/workspace_anthropic.rs"]
mod workspace_anthropic;

#[path = "vertex_ambient/vertex_ambient.rs"]
mod vertex_ambient;

#[path = "env_missing/env_missing.rs"]
mod env_missing;
