use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A graph of one node is the `start` alone: the node runs once and its answer is the result.
#[tokio::test]
async fn one_node_graph() {
    let (p, seen) = program(here(), "one_node_graph", echo).await;
    let out = p.run("hi").await.unwrap();
    assert_eq!(items(&out), [pair("a", "Q: hi")]);
    assert_eq!(seen.lock().unwrap().len(), 1, "one call to the model");
}
