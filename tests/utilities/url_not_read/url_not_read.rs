use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A media from a url is not read yet: with utilities too, the node fails when it runs, and the utility is never
/// called.
#[tokio::test]
async fn url_not_read() {
    let (utility, got) = listen(|_| Resp::ok(r#"{"text":"converted"}"#)).await;
    let (p, _) = program_with(
        here(),
        "url_not_read",
        &[("utility", utility)],
        Resp::ok(chat("ok", None)),
    )
    .await;
    let e = p.run_node(entity(&p), user("x")).await.unwrap_err();
    assert_eq!(e.rule.code(), "io");
    assert_eq!(e.message, "media `doc` is a url, which is not read yet");
    assert!(got.lock().unwrap().is_empty(), "the utility is never called");
}
