use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A markdown media is read and goes with the question.
#[tokio::test]
async fn media_markdown() {
    let (p, seen) = program(here(), "media_markdown", Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("summarise")).await.unwrap();
    assert_eq!(sent(&seen)["messages"][0]["content"], "# the document\n\nsummarise");
}
