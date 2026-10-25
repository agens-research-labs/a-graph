use crate::common::run_react::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A controller that is done without an answer is an error.
#[tokio::test]
async fn no_answer() {
    let (p, _) = program(here(), "no_answer", script(|_, _| json!({ "done": true }))).await;
    assert_eq!(p.run("hi").await.unwrap_err().rule.code(), "missing_field");
}
