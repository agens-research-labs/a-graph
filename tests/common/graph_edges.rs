//! The edges: the list as written, nested, with the nodes that appear more than once.

pub use a_graph::graph::Nodes;
pub use a_graph::graph::edges::{Edges, Pattern, Step};

pub use super::graph::{build as build_nodes, edges_value};

pub fn nodes(src: &str) -> Nodes {
    build_nodes(src).unwrap()
}

pub fn build(src: &str) -> Result<(Nodes, Edges), Vec<(String, String)>> {
    let n = nodes(src);
    Edges::build(Some(edges_value(src)), &n).map(|e| (n, e)).map_err(|e| {
        e.0.iter()
            .map(|e| (e.rule.code().to_string(), e.path.to_string()))
            .collect()
    })
}
