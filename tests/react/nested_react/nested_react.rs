use crate::common::run_react::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A react inside a react is refused before any node runs.
#[tokio::test]
async fn nested_react() {
    let (p, seen) = program(here(), "nested_react", script(|_, _| json!({ "done": true, "answer": "x" }))).await;
    let e = p.run("hi").await.unwrap_err();
    assert_eq!(e.rule.code(), "not_supported");
    assert!(seen.lock().unwrap().is_empty(), "no node ran");
}
