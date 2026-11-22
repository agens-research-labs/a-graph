use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// With `schema`, an answer that is not JSON stops the node: the message names `schema_prompt`, and what
/// the model wrote is the detail, never the message.
#[tokio::test]
async fn schema_text_answer() {
    let answer = "The result is 6, I hope this helps";
    let (p, _) = program(here(), "schema_text_answer", Resp::ok(chat(answer, None))).await;
    let e = p.run_node(entity(&p), user("q")).await.unwrap_err();
    assert_eq!(e.rule.code(), "schema_mismatch");
    assert!(e.message.contains("schema_prompt"), "{}", e.message);
    assert!(!e.message.contains("hope"), "{}", e.message);
    assert!(e.detail.as_deref().is_some_and(|d| d.contains("hope")), "{e:?}");
}
