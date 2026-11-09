use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// What a failing utility wrote on its standard error stays out of the message and, when the
/// passport forbids contents, out of the log; the error keeps it as its detail.
#[tokio::test]
async fn stderr_kept_out() {
    crate::common::clean_output("target/test-output/stderr_kept_out.log");
    let (p, _) = program(here(), "stderr_kept_out", echo).await;
    let e = p.run("hi").await.unwrap_err();
    assert_eq!(e.rule.code(), "utility_failed");
    assert!(!e.message.contains("diabetes"), "{}", e.message);
    assert_eq!(e.detail.as_deref(), Some("the patient has diabetes"));

    let log = std::fs::read_to_string("target/test-output/stderr_kept_out.log").unwrap();
    assert!(!log.contains("diabetes"), "{log}");
    let lines = crate::common::run::log_lines("stderr_kept_out");
    let error = lines.iter().find(|l| l["code"] == "utility_failed").unwrap();
    assert_eq!(error["content_redacted"], true, "{error}");
}
