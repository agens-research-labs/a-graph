use crate::common::graph::*;
use crate::common::{own_yml};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A regular node without validation node is not affected.
#[test]
fn plain_node_unaffected() {
    assert!(build(&own_yml(here())).is_ok());
}
