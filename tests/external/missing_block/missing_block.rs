use crate::common::external::*;

/// A missing block gives empty managers.
#[test]
fn missing_block() {
    let e = External::build(None).unwrap();
    assert!(e.vectordb.iter().next().is_none() && e.search.iter().next().is_none() && e.memory.iter().next().is_none());
}
