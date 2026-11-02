use crate::common::run_graph::*;
use crate::common::trace::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// At the end of the run one POST of OTLP in JSON goes to the endpoint, with the headers of the yml.
#[tokio::test]
async fn sends_otlp_json() {
    let (collector_port, collector) = listen(|_| Resp::ok("{}")).await;
    let (p, _) = program_with(here(), "sends_otlp_json", &[("collector", collector_port)], echo).await;
    p.run("hi").await.unwrap();

    let sent = collector.lock().unwrap();
    assert_eq!(sent.len(), 1, "one POST for the whole run");
    assert_eq!((sent[0].method.as_str(), sent[0].path.as_str()), ("POST", "/v1/traces"));
    assert!(
        sent[0].headers.contains("content-type: application/json") && sent[0].headers.contains("authorization: t"),
        "{}",
        sent[0].headers
    );
    std::fs::create_dir_all("target/test-output").unwrap();
    std::fs::write("target/test-output/otlp_body.json", &sent[0].body).unwrap();
    let body: Value = serde_json::from_str(&sent[0].body).unwrap();
    assert_eq!(
        body["resourceSpans"][0]["resource"]["attributes"][0],
        json!({ "key": "service.name", "value": { "stringValue": "t" } }),
        "the name of the passport"
    );
    drop(sent);

    let spans = spans_of(&collector);
    for name in ["run", "parallel", "linear", "node s", "node d", "node mm", "model m"] {
        assert!(spans.iter().any(|s| s["name"] == name), "no span `{name}`");
    }
    let trace: std::collections::HashSet<&str> = spans.iter().map(|s| s["traceId"].as_str().unwrap()).collect();
    assert_eq!(trace.len(), 1, "one trace");
    assert_eq!(
        spans
            .iter()
            .map(|s| s["spanId"].as_str().unwrap())
            .collect::<std::collections::HashSet<_>>()
            .len(),
        spans.len(),
        "every span has its own id"
    );
    // the data is public: the texts are attributes of the call
    assert_eq!(
        attribute(named(&spans, "model m"), "a_graph.prompt"),
        Some(json!({ "stringValue": "S(hi)" }))
    );
}
