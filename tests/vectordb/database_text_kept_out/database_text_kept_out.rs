use crate::common::run_graph::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// What a vectordb answered with its error stays out of the message and, when the passport forbids
/// contents, out of the log; the error keeps it as its detail.
#[tokio::test]
async fn database_text_kept_out() {
    crate::common::clean_output("target/test-output/database_text_kept_out.log");
    let (embedding_port, _) = listen(|_| Resp::ok(r#"{"embeddings":[[0.5,0.25,1.0]]}"#)).await;
    let (vectordb_port, _) = listen(|_| Resp::status(500, "the patient has diabetes")).await;
    let (p, _) = program_with(
        here(),
        "database_text_kept_out",
        &[("embedding", embedding_port), ("vectordb", vectordb_port)],
        echo,
    )
    .await;
    let e = p.run("hi").await.unwrap_err();
    assert_eq!(e.rule.code(), "io");
    assert!(
        e.message.contains("500") && !e.message.contains("diabetes"),
        "{}",
        e.message
    );
    assert_eq!(e.detail.as_deref(), Some("the patient has diabetes"));

    let log = std::fs::read_to_string("target/test-output/database_text_kept_out.log").unwrap();
    assert!(!log.contains("diabetes"), "{log}");
    let lines = crate::common::run::log_lines("database_text_kept_out");
    let error = lines.iter().find(|l| l["code"] == "io").unwrap();
    assert_eq!(error["content_redacted"], true, "{error}");
}
