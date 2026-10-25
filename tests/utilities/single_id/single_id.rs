use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// One id alone is a list of one.
#[tokio::test]
async fn single_id() {
    let (p, seen) = program(here(), "single_id", Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("hello")).await.unwrap();
    assert_eq!(sent(&seen)["messages"][1]["content"], "Q: HELLO");
}
