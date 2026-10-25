use crate::common::graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// On error is stop or node.
#[test]
fn on_error_words() {
    let n = nodes_of(here(), "on_error_words").unwrap();
    assert_eq!(n.validation().get(n.entity("judge").unwrap()).unwrap().on_error, OnError::Node);
}
