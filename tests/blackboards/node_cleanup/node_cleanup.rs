use crate::common::run_node::*;
use crate::common::{fresh_copy};

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Cleanup empties the board once the task is done.
#[tokio::test]
async fn node_cleanup() {
    let path = "target/test-boards/run_node_cleanup.md";
    fresh_copy(here(), "board.md", path);
    std::fs::write(path, format!("{HEADER}\nold note\n")).unwrap();
    let (p, _) = program(here(), "node_cleanup", Resp::ok(chat("x", None))).await;
    p.run_node(entity(&p), user("q")).await.unwrap();
    assert_eq!(std::fs::read_to_string(path).unwrap(), HEADER);
}
