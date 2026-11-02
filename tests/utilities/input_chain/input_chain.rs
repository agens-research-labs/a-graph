use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// The incoming text goes through the utilities in order, and the node uses the result.
#[tokio::test]
async fn input_chain() {
    let (p, seen) = program(here(), "input_chain", Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("hello")).await.unwrap();
    assert_eq!(
        sent(&seen)["messages"][1]["content"],
        "Q: [HELLO]",
        "first upper case, then in brackets"
    );
}
