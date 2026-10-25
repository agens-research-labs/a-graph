use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A role the node does not read does not reach the model.
#[tokio::test]
async fn read_roles() {
    let (p, seen) = program(here(), "read_roles", Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("new")).await.unwrap();
    let contents: Vec<String> = sent(&seen)["messages"].as_array().unwrap().iter().skip(1).map(|m| m["content"].as_str().unwrap().into()).collect();
    assert_eq!(contents, ["q2", "q3", "Question: new"]);
}
