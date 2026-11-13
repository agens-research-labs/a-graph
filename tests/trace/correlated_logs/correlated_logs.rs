use crate::common::run_graph::*;
use crate::common::trace::*;

fn here() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

/// Logs and OTLP share run/span identities, even when a Program is reused.
#[tokio::test]
async fn correlated_logs() {
    crate::common::clean_output("target/test-output/correlated_logs.log");
    let (port, collector) = listen(|_| Resp::ok("{}")).await;
    let (p, _) = program_with(here(), "correlated_logs", &[("collector", port)], echo).await;
    p.run("hi").await.unwrap();
    p.run("again").await.unwrap();
    let spans = spans_of(&collector);
    let logs = crate::common::run::log_lines("correlated_logs");
    let starts: Vec<_> = logs.iter().filter(|l| l["code"] == "run.start").collect();
    assert_eq!(starts.len(), 2);
    assert_ne!(starts[0]["trace_id"], starts[1]["trace_id"]);
    for log in &logs {
        let trace = log["trace_id"].as_str().unwrap();
        assert_eq!(trace.len(), 32);
        assert!(
            spans
                .iter()
                .any(|s| s["traceId"] == trace && s["spanId"] == log["span_id"]),
            "uncorrelated log: {log}"
        );
    }
    for span in &spans {
        let attributes = span["attributes"].as_array().unwrap();
        let keys: std::collections::HashSet<_> = attributes.iter().map(|a| a["key"].as_str().unwrap()).collect();
        assert_eq!(keys.len(), attributes.len(), "duplicate attributes: {span}");
    }
    let call = named(&spans, "model m");
    assert_eq!(
        attribute(call, "gen_ai.provider.name"),
        Some(json!({"stringValue": "ollama"}))
    );
    assert_eq!(
        attribute(call, "gen_ai.response.model"),
        Some(json!({"stringValue": "m"}))
    );
    assert_eq!(
        attribute(call, "gen_ai.usage.input_tokens"),
        Some(json!({"intValue": "1"}))
    );
    assert_eq!(
        attribute(call, "gen_ai.usage.output_tokens"),
        Some(json!({"intValue": "1"}))
    );
    assert_eq!(
        attribute(call, "a_graph.system_prompt"),
        Some(json!({"stringValue": "Answer briefly."}))
    );
    let log = logs.iter().find(|l| l["code"] == "model.call").unwrap();
    assert_eq!(log["system_prompt"], "Answer briefly.");
    assert_eq!(log["gen_ai.provider.name"], "ollama");
}
