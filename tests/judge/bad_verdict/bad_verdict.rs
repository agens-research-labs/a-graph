use crate::common::run_judge::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A verdict without validation is an error of the judge.
#[tokio::test]
async fn bad_verdict() {
    let (p, _) = program(here(), "bad_verdict", json!({ "reasons": "?", "log": "" })).await;
    let e = p.run("hi").await.unwrap_err();
    assert_eq!((e.rule.code(), e.path.to_string()), ("bad_value", "node(judge)".to_string()));
}
