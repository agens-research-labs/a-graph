use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Each message read and the summary go through the utility of the read.
#[tokio::test]
async fn memory_read_utility() {
    let (p, seen) = program(here(), "memory_read_utility", Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("new")).await.unwrap();
    let messages = sent(&seen)["messages"].clone();
    let contents: Vec<&str> = messages.as_array().unwrap().iter().skip(1).map(|m| m["content"].as_str().unwrap()).collect();
    assert_eq!(contents, ["EARLIER QUESTION", "EARLIER ANSWER", "Q: new / S: EARLIER SUMMARY"]);
}
