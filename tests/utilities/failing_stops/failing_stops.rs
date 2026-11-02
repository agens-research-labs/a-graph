use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A utility that fails stops the node with its own message; the model is never called.
#[tokio::test]
async fn failing_stops() {
    let (p, seen) = program(here(), "failing_stops", Resp::ok(chat("ok", None))).await;
    let e = p.run_node(entity(&p), user("x")).await.unwrap_err();
    assert_eq!(e.rule.code(), "utility_failed");
    assert!(
        e.message.contains("utility `bad`") && e.message.contains("cannot do it"),
        "{}",
        e.message
    );
    assert!(seen.lock().unwrap().is_empty(), "no call to the model");
}
