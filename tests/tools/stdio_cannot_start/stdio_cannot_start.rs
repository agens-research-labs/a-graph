use crate::common::run_tools::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A program that cannot start is a mistake of the configuration.
#[tokio::test]
async fn stdio_cannot_start() {
    let (p, _) = program(here(), "stdio_cannot_start", model(city(), false)).await;
    let e = p.run("hi").await.unwrap_err();
    assert_eq!((e.rule.code(), e.path.to_string()), ("io", "node(n)".to_string()));
    assert!(
        e.message.contains("tool `t`") && e.message.contains("/nonexistent/a-graph-tool"),
        "{}",
        e.message
    );
}
