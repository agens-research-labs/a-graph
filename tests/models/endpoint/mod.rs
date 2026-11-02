//! The host each model is reached at: what the network permission checks. Every case is a yml
//! folder with its own yml files.

use super::common::yml_in;
use a_graph::loader::load;
use a_graph::models::{Model, ModelManager};
use a_graph::schema::key;

fn models_of(src: &str) -> ModelManager {
    // SAFETY: the values are never used to call anything.
    unsafe {
        std::env::set_var("ANTHROPIC_API_KEY", "a");
        std::env::set_var("OPENAI_API_KEY", "o");
    }
    let mut doc = load(src).unwrap_or_else(|e| panic!("{e}"));
    ModelManager::build(doc.take(key::MODELS)).unwrap_or_else(|e| panic!("{e}"))
}

/// The models of `<dir>/<name>.yml`, a file of the test whose folder is `dir`.
fn models(dir: &std::path::Path, name: &str) -> ModelManager {
    models_of(&yml_in(dir, name))
}

fn host(m: &ModelManager, id: &str) -> Result<String, String> {
    m.spec(id).map(Model::host).unwrap()
}

#[path = "file_model_hosts/file_model_hosts.rs"]
mod file_model_hosts;

#[path = "api_key_hosts/api_key_hosts.rs"]
mod api_key_hosts;

#[path = "vertex_host/vertex_host.rs"]
mod vertex_host;

#[path = "azure_host/azure_host.rs"]
mod azure_host;

#[path = "local_host_url/local_host_url.rs"]
mod local_host_url;
