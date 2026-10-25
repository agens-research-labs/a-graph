use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A file where the place needs a text is an error that says so.
#[tokio::test]
async fn file_in_text_place() {
    let (p, _) = program(here(), "file_in_text_place", Resp::ok(chat("ok", None))).await;
    let e = p.run_node(entity(&p), user("x")).await.unwrap_err();
    assert_eq!(e.rule.code(), "utility_failed");
    assert!(e.message.contains("answered a file (application/pdf)") && e.message.contains("message.output"), "{}", e.message);
}
