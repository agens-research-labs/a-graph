use crate::common::graph::*;

/// A missing list gives no nodes.
#[test]
fn missing_list() {
    assert!(Nodes::build(None).unwrap().is_empty());
}
