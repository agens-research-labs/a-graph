use crate::common::run_judge::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A verdict without validation fails the run without its text: the message and the log keep it out
/// when the passport forbids contents, and the error keeps it as its detail.
#[tokio::test]
async fn verdict_kept_out() {
    crate::common::clean_output("target/test-output/verdict_kept_out.log");
    let verdict = json!({ "reasons": "the patient has diabetes", "log": "" });
    let (p, _) = program(here(), "verdict_kept_out", verdict).await;
    let e = p.run("hi").await.unwrap_err();
    assert_eq!(e.rule.code(), "schema_mismatch");
    assert!(!e.message.contains("diabetes"), "{}", e.message);
    assert!(e.detail.as_deref().is_some_and(|d| d.contains("diabetes")), "{e:?}");

    let log = std::fs::read_to_string("target/test-output/verdict_kept_out.log").unwrap();
    assert!(!log.contains("diabetes"), "{log}");
    let lines = crate::common::run::log_lines("verdict_kept_out");
    let error = lines.iter().find(|l| l["code"] == "schema_mismatch").unwrap();
    assert_eq!(error["content_redacted"], true, "{error}");
}
