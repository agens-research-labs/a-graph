use crate::common::run_graph::*;
use crate::common::trace::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Content capture is opt-in, bounded on UTF-8 boundaries, and never overrides the passport.
#[tokio::test]
async fn content_policy() {
    for (settings, private, expected) in [
        ("", false, None),
        ("  capture_content: false\n", false, None),
        ("  capture_content: true\n  max_content_bytes: 5\n", false, Some("éé")),
        ("  capture_content: true\n", true, None),
    ] {
        let (collector_port, collector) = listen(|_| Resp::ok("{}")).await;
        let (model_port, _) = listen(echo).await;
        let source = yml_with(
            here(),
            "content_policy",
            &[("collector", collector_port), ("model", model_port)],
        );
        let source = source.replace("trace:\n", &format!("trace:\n{settings}"));
        let source = if private {
            source.replace("sensitivity: public", "sensitivity: confidential")
        } else {
            source
        };
        let p = Arc::new(build(&source).unwrap());
        p.run("ééé-secret").await.unwrap();
        let spans = spans_of(&collector);
        let call = named(&spans, "model m");
        assert_eq!(call["kind"], 3);
        assert_eq!(
            attribute(call, "gen_ai.request.model"),
            Some(json!({"stringValue": "m"}))
        );
        assert_eq!(
            attribute(call, "a_graph.prompt"),
            expected.map(|s| json!({"stringValue": s}))
        );
        if expected.is_some() {
            assert_eq!(
                attribute(call, "a_graph.content_truncated"),
                Some(json!({"boolValue": true}))
            );
        } else {
            assert_eq!(
                attribute(call, "a_graph.content_redacted"),
                Some(json!({"boolValue": true}))
            );
            assert!(attribute(call, "a_graph.system_prompt").is_none());
        }
        let sent = collector.lock().unwrap();
        let body: Value = serde_json::from_str(&sent[0].body).unwrap();
        let resource = &body["resourceSpans"][0];
        let attrs = resource["resource"]["attributes"].as_array().unwrap();
        assert!(
            attrs
                .iter()
                .any(|a| a["key"] == "service.version" && a["value"]["stringValue"] == env!("CARGO_PKG_VERSION"))
        );
        assert!(
            attrs
                .iter()
                .any(|a| a["key"] == "a_graph.graph.version" && a["value"]["stringValue"] == "2.3.4")
        );
        assert_eq!(resource["scopeSpans"][0]["scope"]["version"], env!("CARGO_PKG_VERSION"));
    }
}

/// Opted-in contents are bounded even when no explicit limit is configured.
#[tokio::test]
async fn default_content_limit() {
    let (port, collector) = listen(|_| Resp::ok("{}")).await;
    let (model_port, _) = listen(echo).await;
    let source = yml_with(here(), "content_policy", &[("collector", port), ("model", model_port)])
        .replace("trace:\n", "trace:\n  capture_content: true\n");
    let p = Arc::new(build(&source).unwrap());
    p.run(&"x".repeat(5000)).await.unwrap();
    let spans = spans_of(&collector);
    let call = named(&spans, "model m");
    let prompt = attribute(call, "a_graph.prompt").unwrap();
    assert_eq!(prompt["stringValue"].as_str().unwrap().len(), 4096);
    assert_eq!(
        attribute(call, "a_graph.content_truncated"),
        Some(json!({"boolValue": true}))
    );
}

#[tokio::test]
async fn rejects_zero_limit() {
    let source = yml_with(here(), "content_policy", &[("collector", 1), ("model", 2)])
        .replace("trace:\n", "trace:\n  max_content_bytes: 0\n");
    let errors = build(&source).err().expect("zero content limit must fail");
    assert!(errors.0.iter().any(|e| e.path.to_string() == "trace.max_content_bytes"));
}
