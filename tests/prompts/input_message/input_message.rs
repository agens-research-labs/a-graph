use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Input message is the one message the node takes.
#[tokio::test]
async fn input_message() {
    let (p, seen) = program(here(), "input_message", Resp::ok(chat("ok", None))).await;
    p.run_node(
        entity(&p),
        NodeInput {
            agent_message: Some("A says".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let body = sent(&seen);
    assert_eq!(
        (
            body["messages"][0]["content"].as_str(),
            body["messages"][1]["content"].as_str()
        ),
        (Some("S A says"), Some("U A says"))
    );
}
