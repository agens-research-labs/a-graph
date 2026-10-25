use crate::common::run_graph::*;
use crate::common::trace::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// With a data classification that forbids contents the spans keep the facts and drop the texts.
#[tokio::test]
async fn content_redacted() {
    let (_, collector) = serve_at(19816, |_| Resp::ok("{}")).await;
    let (p, _) = program(here(), "content_redacted", echo).await;
    p.run("hi").await.unwrap();
    let spans = spans_of(&collector);
    let call = named(&spans, "model m");
    assert_eq!(attribute(call, "a_graph.content_redacted"), Some(json!({ "boolValue": true })));
    assert!(attribute(call, "a_graph.prompt").is_none() && attribute(call, "a_graph.answer").is_none());
    assert_eq!(attribute(call, "a_graph.input_tokens"), Some(json!({ "intValue": "1" })));
}
