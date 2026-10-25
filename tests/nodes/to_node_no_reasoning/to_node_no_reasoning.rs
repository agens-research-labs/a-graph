use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The answer goes to the connected node without the reasoning.
#[tokio::test]
async fn to_node_no_reasoning() {
    let (p, _) = program(here(), "to_node_no_reasoning", Resp::ok(chat("the answer", Some("I thought")))).await;
    let out = p.run_node(entity(&p), user("q")).await.unwrap();
    assert_eq!(out.node.message, Some(Content::Text("the answer".into())));
    assert!(out.node.reasoning.is_empty(), "the reasoning never goes to a node");
    assert!(out.assistant.message.is_none());
}
