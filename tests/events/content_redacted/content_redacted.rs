use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// With a data classification that forbids contents the event keeps its facts and drops the texts.
#[tokio::test]
async fn content_redacted() {
    crate::common::clean_output("target/test-output/content_redacted.log");
    let (p, _) = program(here(), "content_redacted", echo).await;
    p.run("hi").await.unwrap();
    let lines = crate::common::run::log_lines("content_redacted");
    let call = lines.iter().find(|l| l["code"] == "model.call").unwrap();
    assert_eq!(call["content_redacted"], true);
    assert!(call.get("prompt").is_none() && call.get("answer").is_none());
    assert_eq!(call["input_tokens"], 1);
}
