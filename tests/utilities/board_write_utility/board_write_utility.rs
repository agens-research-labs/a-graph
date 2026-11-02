use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The blackboard gets the text as the node delivered it, changed once more by its own utility.
#[tokio::test]
async fn board_write_utility() {
    let (p, _) = program(here(), "board_write_utility", Resp::ok(chat("note", None))).await;
    let out = p.run_node(entity(&p), user("q")).await.unwrap();
    assert_eq!(
        out.assistant.message,
        Some(Content::Text("NOTE".into())),
        "the output has only the first change"
    );
    assert!(
        p.blackboards.read("bb").unwrap().contains("[NOTE]"),
        "the board has both"
    );
}
