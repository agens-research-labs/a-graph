use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The answer goes to the final output with its reasoning.
#[tokio::test]
async fn to_output_reasoning() {
    let (p, _) = program(here(), "to_output_reasoning", Resp::ok(chat("the answer", Some("I thought")))).await;
    let out = p.run_node(entity(&p), user("q")).await.unwrap();
    assert_eq!(out.assistant.message, Some(Content::Text("the answer".into())));
    assert_eq!(out.assistant.reasoning, vec![json!({ "thinking": "I thought" })], "the reasoning goes to the final output");
    assert!(out.node.message.is_none() && out.node.reasoning.is_empty(), "nothing goes to the connected node");
}
