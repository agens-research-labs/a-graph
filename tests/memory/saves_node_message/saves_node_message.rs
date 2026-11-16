use crate::common::clean_output;
use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node that takes only the message of the node before saves that message, not the one of the user.
#[tokio::test]
async fn saves_node_message() {
    let path = "target/test-memory/saves_node_message.json";
    clean_output(path);
    let (p, _) = program(here(), "saves_node_message", Resp::ok(chat("the answer", None))).await;
    let input = NodeInput {
        user_message: Some("the text of the user".into()),
        agent_message: Some("the text of the node before".into()),
        ..Default::default()
    };
    p.run_node(entity(&p), input).await.unwrap();
    let records: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(
        records[0]["chat"],
        json!([
            { "role": "user", "content": "the text of the node before" },
            { "role": "assistant", "content": "the answer" }
        ])
    );
}
