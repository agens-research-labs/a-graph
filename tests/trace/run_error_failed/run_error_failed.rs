use crate::common::run_graph::*;
use crate::common::trace::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// A run that fails still sends its trace, with the run marked as failed and why.
#[tokio::test]
async fn run_error_failed() {
    let (collector_port, collector) = listen(|_| Resp::ok("{}")).await;
    let (p, _) = program_with(here(), "run_error_failed", &[("collector", collector_port)], |_| {
        Resp::status(500, "broken")
    })
    .await;
    assert!(p.run("hi").await.is_err());
    let spans = spans_of(&collector);
    let run = named(&spans, "run");
    assert_eq!(run["status"]["code"], 2);
    assert!(
        run["status"]["message"].as_str().unwrap().contains("model_failed"),
        "{run}"
    );
    let call = named(&spans, "model m");
    assert_eq!(call["kind"], 3);
    assert_eq!(
        attribute(call, "gen_ai.request.model"),
        Some(json!({"stringValue": "requested-model"}))
    );
    assert_eq!(call["status"]["code"], 2);
    assert_eq!(attribute(call, "error.type"), Some(json!({"stringValue": "provider"})));
    assert_eq!(
        attribute(call, "http.response.status_code"),
        Some(json!({"intValue": "500"}))
    );
    assert_eq!(
        attribute(call, "gen_ai.provider.name"),
        Some(json!({"stringValue": "ollama"}))
    );
    assert!(attribute(call, "a_graph.detail").is_none(), "trace content is opt-in");
    assert!(end(call) >= start(call));
    let node = named(&spans, "node s");
    assert_eq!(call["parentSpanId"], node["spanId"]);
    assert!(
        node["status"]["message"].as_str().unwrap().contains("model.error"),
        "{node}"
    );
}

/// Preserve the provider adapter's error classification, including malformed HTTP successes.
/// Ollama reports a 401 as Provider, not Credentials: tracing must not reinterpret that contract.
#[tokio::test]
async fn error_kinds() {
    for (status, body, kind) in [(401, "unauthorized", "provider"), (200, "not JSON", "invalid_response")] {
        let (port, collector) = listen(|_| Resp::ok("{}")).await;
        let (p, _) = program_with(here(), "run_error_failed", &[("collector", port)], move |_| {
            Resp::status(status, body)
        })
        .await;
        assert!(p.run("hi").await.is_err());
        let spans = spans_of(&collector);
        let call = named(&spans, "model m");
        assert_eq!(call["status"]["code"], 2);
        assert_eq!(attribute(call, "error.type"), Some(json!({"stringValue": kind})));
        assert_eq!(
            attribute(call, "gen_ai.request.model"),
            Some(json!({"stringValue": "requested-model"}))
        );
    }
}
