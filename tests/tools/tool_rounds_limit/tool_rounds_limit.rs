use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Requests no `max_iter` counts (a tool the node does not have) still stop at the `max_rounds` of the node.
#[tokio::test]
async fn tool_rounds_limit() {
    let (p, seen) = program(here(), "tool_rounds_limit", model(vec![("nope", json!({}))], true)).await;
    let e = p.run("hi").await.unwrap_err();
    assert_eq!((e.rule.code(), e.path.to_string()), ("tool_limit", "node(n)".to_string()));
    assert!(e.message.contains("`max_rounds`") && e.message.contains("2 rounds"), "{}", e.message);
    assert_eq!(seen.lock().unwrap().len(), 3, "the third round was refused");
}
