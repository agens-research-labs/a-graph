use crate::common::run_graph::*;
use crate::common::verbose::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// When the data classification forbids contents, `detail` says they are hidden and shows none.
#[tokio::test]
async fn content_hidden() {
    let (mut p, _) = program(here(), "content_hidden", echo).await;
    let screen = show(&mut p, VerboseLevel::Detail, false, false);
    p.run("hi").await.unwrap();
    let text = screen.text();
    assert!(text.contains("contents hidden"), "{text}");
    assert!(!text.contains("prompt:") && !text.contains("answer:"), "{text}");
    assert!(text.contains("model m ·"), "the facts stay: {text}");
}
