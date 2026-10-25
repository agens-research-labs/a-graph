use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node that reads its board gets the text of the board (header included) where the prompt has {{bb}}.
#[tokio::test]
async fn board_read_fills() {
    let path = "target/test-boards/board_read_fills.md";
    std::fs::create_dir_all("target/test-boards").unwrap();
    std::fs::write(path, format!("{HEADER}\nThe lease lasts 12 months.\n")).unwrap();
    let (p, seen) = program(here(), "board_read_fills", Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("how long?")).await.unwrap();
    assert_eq!(sent(&seen)["messages"][1]["content"], format!("Notes: {HEADER}\nThe lease lasts 12 months.\n / Question: how long?"));
}
