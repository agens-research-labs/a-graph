use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// With `schema`, a JSON that does not match the schema stops the node; the reasons of the validator are the
/// detail, never the message.
#[tokio::test]
async fn schema_wrong_shape() {
    let (p, _) = program(
        here(),
        "schema_wrong_shape",
        Resp::ok(chat("{\"b\": \"secret\"}", None)),
    )
    .await;
    let e = p.run_node(entity(&p), user("q")).await.unwrap_err();
    assert_eq!(e.rule.code(), "schema_mismatch");
    assert!(!e.message.contains("secret"), "{}", e.message);
    assert!(e.detail.as_deref().is_some_and(|d| d.contains("required")), "{e:?}");

    let (p, _) = program(here(), "schema_wrong_shape", Resp::ok(chat("{\"a\": 3}", None))).await;
    let out = p.run_node(entity(&p), user("q")).await.unwrap();
    assert_eq!(out.assistant.message, Some(Content::Json(json!({ "a": 3 }))));
}
