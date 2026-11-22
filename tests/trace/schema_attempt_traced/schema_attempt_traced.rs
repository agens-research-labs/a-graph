use crate::common::run_graph::*;
use crate::common::trace::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A checked answer is a point inside the interval of its node, with its fields as attributes.
#[tokio::test]
async fn schema_attempt_traced() {
    let (collector_port, collector) = listen(|_| Resp::ok("{}")).await;
    let (p, _) = program_with(
        here(),
        "schema_attempt_traced",
        &[("collector", collector_port)],
        |_| Resp::ok(chat("{\"a\": 1}")),
    )
    .await;
    p.run("hi").await.unwrap();
    let spans = spans_of(&collector);
    let node = named(&spans, "node n");
    let attempt = named(&spans, "structured valid");
    assert_eq!(attempt["parentSpanId"], node["spanId"], "{attempt}");
    let text = |key: &str| attribute(attempt, key).and_then(|v| v["stringValue"].as_str().map(str::to_string));
    assert_eq!(
        (text("a_graph.mechanism"), text("a_graph.outcome")),
        (Some("api".into()), Some("valid".into())),
        "{attempt}"
    );
}
