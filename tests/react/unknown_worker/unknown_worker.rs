use crate::common::run_react::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A worker that does not exist is an error.
#[tokio::test]
async fn unknown_worker() {
    let (p, _) = program(
        here(),
        "unknown_worker",
        script(|_, _| json!({ "done": false, "next_worker": "ghost", "input": "x" })),
    )
    .await;
    let e = p.run("hi").await.unwrap_err();
    assert_eq!(
        (e.rule.code(), e.path.to_string()),
        ("unknown_ref", "node(c)".to_string())
    );
}
