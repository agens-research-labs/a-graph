use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The utilities of `message.input` change only the messages the node takes: here not the one of the user.
#[tokio::test]
async fn skips_user_text() {
    let (p, seen) = program(here(), "skips_user_text", Resp::ok(chat("ok", None))).await;
    let input = NodeInput {
        user_message: Some("the text of the user".into()),
        agent_message: Some("the text of the node before".into()),
        ..Default::default()
    };
    p.run_node(entity(&p), input).await.unwrap();
    assert_eq!(
        sent(&seen)["messages"][1]["content"],
        "From the node before: THE TEXT OF THE NODE BEFORE"
    );
}
