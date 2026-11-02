use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A tool asked more than its max iter stops the run.
#[tokio::test]
async fn tool_over_max_iter() {
    let (p, seen) = program(here(), "tool_over_max_iter", model(city(), true)).await;
    let e = p.run("hi").await.unwrap_err();
    assert_eq!(
        (e.rule.code(), e.path.to_string()),
        ("tool_limit", "node(n)".to_string())
    );
    assert!(
        e.message.contains("`t`") && e.message.contains("2 times"),
        "{}",
        e.message
    );
    assert_eq!(
        seen.lock().unwrap().len(),
        3,
        "the third request of the tool was refused"
    );
}
