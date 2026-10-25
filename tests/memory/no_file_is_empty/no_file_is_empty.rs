use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A memory that was never written is empty, not an error.
#[tokio::test]
async fn no_file_is_empty() {
    let (p, seen) = program(here(), "no_file_is_empty", Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("new")).await.unwrap();
    let n = sent(&seen)["messages"].as_array().unwrap().len();
    assert_eq!(n, 2, "system and the new message only");
}
