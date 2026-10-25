use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Without a prompt, a node with one message gives it to the model as it is.
#[tokio::test]
async fn no_prompt_text() {
    let (p, seen) = program(here(), "no_prompt_text", Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("one")).await.unwrap();
    assert_eq!(sent(&seen)["messages"][0]["content"], "one");
}
