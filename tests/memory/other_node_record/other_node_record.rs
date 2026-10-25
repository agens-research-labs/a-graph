use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A node reads only its own record.
#[tokio::test]
async fn other_node_record() {
    let (p, seen) = program(here(), "other_node_record", Resp::ok(chat("ok", None))).await;
    p.run_node(entity(&p), user("new")).await.unwrap();
    assert_eq!(sent(&seen)["messages"].as_array().unwrap().len(), 2);
}
