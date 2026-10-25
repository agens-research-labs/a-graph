use crate::common::graph_edges::*;
use crate::common::{yml_in};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node that appears twice is two places in the list.
#[test]
fn node_twice() {
    // after the first `b` comes `c`, after the second `b` comes `f`: the list keeps both places
    let (n, e) = build(&yml_in(here(), "node_twice")).unwrap();
    let nodes: Vec<&str> = e.steps().iter().map(|s| match s { Step::Node(x) => n.id(*x).unwrap(), _ => panic!() }).collect();
    assert_eq!(nodes, ["b", "c", "b", "f"]);
}
