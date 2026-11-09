use crate::common::run_graph::*;
use crate::common::trace::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// When the passport allows contents, the interval an error ends holds what made the error as the
/// attribute `a_graph.detail`, apart from its status.
#[tokio::test]
async fn error_detail_shown() {
    let (collector_port, collector) = listen(|_| Resp::ok("{}")).await;
    let verdict = json!({ "reasons": "the patient has diabetes", "log": "" });
    let (p, _) = program_with(
        here(),
        "error_detail_shown",
        &[("collector", collector_port)],
        crate::common::run_judge::mock(verdict),
    )
    .await;
    assert!(p.run("hi").await.is_err());
    let spans = spans_of(&collector);
    let run = named(&spans, "run");
    let status = run["status"]["message"].as_str().unwrap();
    assert!(status.contains("bad_value") && !status.contains("diabetes"), "{run}");
    let detail = attribute(run, "a_graph.detail").unwrap_or_default();
    assert!(
        detail["stringValue"].as_str().unwrap_or_default().contains("diabetes"),
        "{run}"
    );
}
