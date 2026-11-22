use crate::common::run_react::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// After max iterations a new worker call is an error.
#[tokio::test]
async fn max_iterations() {
    // max_iterations is 2: two workers are called, the third request is refused
    let (p, seen) = program(
        here(),
        "max_iterations",
        script(|_, _| full(json!({ "done": false, "next_worker": "w1", "input": "again" }))),
    )
    .await;
    let e = p.run("hi").await.unwrap_err();
    assert_eq!(
        (e.rule.code(), e.path.to_string()),
        ("react_limit", "node(c)".to_string())
    );
    assert_eq!(controller_turns(&seen).len(), 3);
}
