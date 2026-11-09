use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// What a provider wrote in its error stays out of the message and, when the passport forbids
/// contents, out of the log; the error keeps it as its detail.
#[tokio::test]
async fn provider_text_kept_out() {
    crate::common::clean_output("target/test-output/provider_text_kept_out.log");
    let (p, _) = program(here(), "provider_text_kept_out", |_| {
        Resp::status(500, json!({ "error": "the patient has diabetes" }).to_string())
    })
    .await;
    let e = p.run("hi").await.unwrap_err();
    assert_eq!(e.rule.code(), "model_failed");
    assert!(
        e.message.contains("500") && !e.message.contains("diabetes"),
        "{}",
        e.message
    );
    assert_eq!(e.detail.as_deref(), Some("the patient has diabetes"));

    let log = std::fs::read_to_string("target/test-output/provider_text_kept_out.log").unwrap();
    assert!(!log.contains("diabetes"), "{log}");
    let lines = crate::common::run::log_lines("provider_text_kept_out");
    let error = lines.iter().find(|l| l["code"] == "model_failed").unwrap();
    assert_eq!(error["content_redacted"], true, "{error}");
}
