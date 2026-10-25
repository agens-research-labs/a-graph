use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A message the node takes must be given.
#[tokio::test]
async fn message_required() {
    let (p, seen) = program(here(), "message_required", Resp::ok(chat("ok", None))).await;
    let e = p.run_node(entity(&p), NodeInput::default()).await.unwrap_err();
    assert_eq!(e.rule.code(), "missing_field");
    assert!(seen.lock().unwrap().is_empty(), "the model was not called");
}
