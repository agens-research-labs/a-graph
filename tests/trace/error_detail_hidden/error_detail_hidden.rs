use crate::common::run_graph::*;
use crate::common::trace::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// When the passport forbids contents, the interval an error ends says so and holds nothing of what
/// made the error, neither in its status nor in its attributes.
#[tokio::test]
async fn error_detail_hidden() {
    let (collector_port, collector) = listen(|_| Resp::ok("{}")).await;
    let verdict = json!({ "reasons": "the patient has diabetes", "log": "" });
    let (p, _) = program_with(
        here(),
        "error_detail_hidden",
        &[("collector", collector_port)],
        crate::common::run_judge::mock(verdict),
    )
    .await;
    assert!(p.run("hi").await.is_err());
    let sent: String = collector.lock().unwrap().iter().map(|r| r.body.clone()).collect();
    assert!(!sent.contains("diabetes"), "{sent}");
    let spans = spans_of(&collector);
    let run = named(&spans, "run");
    assert_eq!(run["status"]["code"], 2);
    assert_eq!(
        attribute(run, "a_graph.content_redacted"),
        Some(json!({ "boolValue": true })),
        "{run}"
    );
    assert!(attribute(run, "a_graph.detail").is_none(), "{run}");
}
