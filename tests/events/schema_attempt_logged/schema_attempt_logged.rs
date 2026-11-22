use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Every answer checked against a schema writes `structured.attempt`: `debug` when valid, `warn` otherwise, with
/// its reasons as contents; `node.end` says how many corrections the node needed, only when it has a schema.
#[tokio::test]
async fn schema_attempt_logged() {
    crate::common::clean_output("target/test-output/schema_attempt_valid.log");
    let (p, _) = program(here(), "valid", |_| Resp::ok(chat("{\"a\": 1}"))).await;
    p.run("hi").await.unwrap();
    let lines = crate::common::run::log_lines("schema_attempt_valid");
    let attempt = lines.iter().find(|l| l["code"] == "structured.attempt").unwrap();
    assert_eq!(
        (
            attempt["level"].as_str(),
            attempt["node"].as_str(),
            attempt["model"].as_str(),
            attempt["mechanism"].as_str(),
            attempt["attempt"].as_u64(),
            attempt["max_attempts"].as_u64(),
            attempt["outcome"].as_str(),
        ),
        (
            Some("debug"),
            Some("n"),
            Some("m"),
            Some("api"),
            Some(1),
            Some(1),
            Some("valid")
        ),
        "{attempt}"
    );
    let end = lines.iter().find(|l| l["code"] == "node.end").unwrap();
    assert_eq!(end["corrections"].as_u64(), Some(0), "{end}");

    crate::common::clean_output("target/test-output/schema_attempt_text.log");
    let (p, _) = program(here(), "text", |_| Resp::ok(chat("six"))).await;
    p.run("hi").await.unwrap_err();
    let lines = crate::common::run::log_lines("schema_attempt_text");
    let attempt = lines.iter().find(|l| l["code"] == "structured.attempt").unwrap();
    assert_eq!(
        (attempt["level"].as_str(), attempt["outcome"].as_str()),
        (Some("warn"), Some("not_json")),
        "{attempt}"
    );
    let message = attempt["message"].as_str().unwrap_or_default();
    assert!(!message.is_empty() && !message.contains("six"), "{attempt}");
    assert!(attempt["reasons"].as_str().is_some_and(|r| !r.is_empty()), "{attempt}");
    // the attempt is a warning: the error of the run comes after it
    let at = |code: &str| lines.iter().position(|l| l["code"] == code).unwrap();
    assert!(at("structured.attempt") < at("schema_mismatch"), "{lines:?}");

    crate::common::clean_output("target/test-output/schema_attempt_none.log");
    let (p, _) = program(here(), "no_schema", |_| Resp::ok(chat("six"))).await;
    p.run("hi").await.unwrap();
    let lines = crate::common::run::log_lines("schema_attempt_none");
    assert!(!lines.iter().any(|l| l["code"] == "structured.attempt"), "{lines:?}");
    let end = lines.iter().find(|l| l["code"] == "node.end").unwrap();
    assert!(end.get("corrections").is_none(), "{end}");
}
