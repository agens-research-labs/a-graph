use crate::common::run_node::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// With `schema`, a JSON left unclosed by a model that ended by itself (not at a limit) is not usable JSON.
#[tokio::test]
async fn schema_broken_answer() {
    let (p, _) = program(here(), "schema_broken_answer", Resp::ok(chat("{\"a\": 1", None))).await;
    let e = p.run_node(entity(&p), user("q")).await.unwrap_err();
    assert_eq!(e.rule.code(), "schema_mismatch");
}
