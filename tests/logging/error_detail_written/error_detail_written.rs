use crate::common::run_judge::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// When the passport allows contents, the log writes what made the error as its `detail`, apart
/// from the `message`.
#[tokio::test]
async fn error_detail_written() {
    crate::common::clean_output("target/test-output/error_detail_written.log");
    let verdict = json!({ "reasons": "the patient has diabetes", "log": "" });
    let (p, _) = program(here(), "error_detail_written", verdict).await;
    p.run("hi").await.unwrap_err();
    let lines = crate::common::run::log_lines("error_detail_written");
    let error = lines.iter().find(|l| l["code"] == "schema_mismatch").unwrap();
    assert_eq!(
        error["message"], "the answer of model `m` (ollama) does not match its schema",
        "{error}"
    );
    let detail = error["detail"].as_str().unwrap_or_default();
    assert!(detail.contains("the patient has diabetes"), "{error}");
    assert!(error.get("content_redacted").is_none(), "{error}");
}
