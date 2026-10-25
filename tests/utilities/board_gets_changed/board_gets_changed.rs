use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node with no destination for its text still changes it before the blackboard gets it.
#[tokio::test]
async fn board_gets_changed() {
    let (p, _) = program(here(), "board_gets_changed", Resp::ok(chat("note", None))).await;
    let out = p.run_node(entity(&p), user("q")).await.unwrap();
    assert!(out.assistant.message.is_none() && out.node.message.is_none(), "nothing goes on");
    assert!(p.blackboards.read("bb").unwrap().contains("[note]"));
}
