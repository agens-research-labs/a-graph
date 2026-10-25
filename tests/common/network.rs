//! The network permission: every test is a folder with its own yml files.


pub use a_graph::blackboards::BlackboardManager;
pub use a_graph::external::External;
pub use a_graph::loader::load;
pub use a_graph::models::ModelManager;
pub use a_graph::network::{allowed_hosts, check, logging_destinations};
pub use a_graph::resources::Resources;
pub use a_graph::schema::key;
pub use super::{yml_in};

/// The (rule, path) pairs the network check reports for this yml.
pub fn errors_of(src: &str) -> Vec<(String, String)> {
    // SAFETY: the values are never used to call anything.
    unsafe {
        std::env::set_var("ANTHROPIC_API_KEY", "a");
        std::env::set_var("OPENAI_API_KEY", "o");
    }
    let mut doc = load(src).unwrap_or_else(|e| panic!("the Loader rejected the yml: {e}"));
    let allowed = allowed_hosts(doc.take(key::PASSPORT)).unwrap();
    let models = ModelManager::build(doc.take(key::MODELS)).unwrap_or_else(|e| panic!("{e}"));
    let resources = Resources::build(doc.take(key::RESOURCES)).unwrap_or_else(|e| panic!("{e}"));
    let external = External::build(doc.take(key::EXTERNAL)).unwrap_or_else(|e| panic!("{e}"));
    let boards = BlackboardManager::build(doc.take(key::BLACKBOARDS)).unwrap_or_else(|e| panic!("{e}"));
    let mut destinations = logging_destinations(doc.take(key::OBSERVABILITY), doc.take(key::TRACE)).unwrap();
    destinations.extend(models.destinations());
    destinations.extend(resources.destinations());
    destinations.extend(external.destinations());
    destinations.extend(boards.destinations());
    check(&allowed, destinations).iter().map(|e| (e.rule.code().to_string(), e.path.to_string())).collect()
}

/// The errors of `<dir>/<name>.yml`, a file of the test whose folder is `dir`.
pub fn errors(dir: &std::path::Path, name: &str) -> Vec<(String, String)> {
    errors_of(&yml_in(dir, name))
}

pub fn refused(dir: &std::path::Path, name: &str, path: &str) -> bool {
    errors(dir, name).iter().any(|(r, p)| r == "network_not_allowed" && p == path)
}









