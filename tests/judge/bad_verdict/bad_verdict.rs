use crate::common::run_judge::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A verdict without validation does not match the schema of the verdict: an error of the judge.
#[tokio::test]
async fn bad_verdict() {
    let (p, _) = program(here(), "bad_verdict", json!({ "reasons": "?", "log": "" })).await;
    let e = p.run("hi").await.unwrap_err();
    assert_eq!(
        (e.rule.code(), e.path.to_string()),
        ("schema_mismatch", "node(judge)".to_string())
    );
}
