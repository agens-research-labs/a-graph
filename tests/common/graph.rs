//! The nodes and the edges of the graph: every test is a folder with its own yml files.



pub use a_graph::blackboards::Place;
pub use a_graph::graph::Nodes;
pub use a_graph::graph::nodes::component::{OnError, Query};
pub use a_graph::loader::load;
pub use a_graph::models::Reasoning;
pub use a_graph::schema::key;
pub use super::{yml_in};

/// `graph.edges` of a yml, as the Loader hands it over.
pub fn edges_value(src: &str) -> serde_norway::Value {
    let mut doc = load(src).unwrap_or_else(|e| panic!("the Loader rejected the yml: {e}"));
    let serde_norway::Value::Mapping(mut graph) = doc.take(key::GRAPH).unwrap() else { panic!("graph is a map") };
    graph.remove(key::EDGES).unwrap()
}

/// `graph.nodes` of a yml, as the Loader hands it over.
pub fn nodes_value(src: &str) -> serde_norway::Value {
    let mut doc = load(src).unwrap_or_else(|e| panic!("the Loader rejected the yml: {e}"));
    let serde_norway::Value::Mapping(mut graph) = doc.take(key::GRAPH).unwrap() else { panic!("graph is a map") };
    graph.remove(key::NODES).unwrap()
}

pub fn build(src: &str) -> Result<Nodes, Vec<(String, String)>> {
    Nodes::build(Some(nodes_value(src))).map_err(|e| e.0.iter().map(|e| (e.rule.code().to_string(), e.path.to_string())).collect())
}

/// The nodes of `<dir>/<name>.yml`, a file of the test whose folder is `dir`.
pub fn nodes_of(dir: &std::path::Path, name: &str) -> Result<Nodes, Vec<(String, String)>> {
    build(&yml_in(dir, name))
}











