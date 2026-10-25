use crate::common::run_node::*;
use crate::common::{fresh_copy};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The node reads writes and cleans its blackboard.
#[tokio::test]
async fn node_board_cycle() {
    let path = "target/test-boards/run_node_rw.md";
    fresh_copy(here(), "board.md", path);
    std::fs::write(path, format!("{HEADER}\nold note\n")).unwrap();
    let (p, seen) = program(here(), "node_board_cycle", Resp::ok(chat("new finding", None))).await;
    p.run_node(entity(&p), user("q")).await.unwrap();

    // what the model saw: the whole board, header included
    assert!(sent(&seen)["messages"][0]["content"].as_str().unwrap().contains("old note"));
    // what the node left: below the header, after the old text
    assert_eq!(std::fs::read_to_string(path).unwrap(), format!("{HEADER}\nold note\n\nnew finding\n"));
}
