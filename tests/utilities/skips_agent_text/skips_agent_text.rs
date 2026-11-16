use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The utilities of `message.input` change only the messages the node takes: here not the one of the node before.
#[tokio::test]
async fn skips_agent_text() {
    let (p, seen) = program(here(), "skips_agent_text", Resp::ok(chat("ok", None))).await;
    let input = NodeInput {
        user_message: Some("the question".into()),
        agent_message: Some("the text of the node before".into()),
        ..Default::default()
    };
    p.run_node(entity(&p), input).await.unwrap();
    assert_eq!(sent(&seen)["messages"][1]["content"], "Q: THE QUESTION");
}
