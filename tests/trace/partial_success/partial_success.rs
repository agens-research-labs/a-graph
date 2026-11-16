use crate::common::cli::*;

/// OTLP partial acceptance and warning messages are reported without replaying accepted spans
/// or echoing untrusted response text. Ordinary JSON log delivery is unaffected.
#[tokio::test]
async fn partial_success() {
    let oversized = "x".repeat(65537);
    for (body, expected) in [
        (
            r#"{"partialSuccess":{"rejectedSpans":"2","errorMessage":"SECRET-REMOTE-CONTENT"}}"#,
            "2 rejected spans",
        ),
        (
            r#"{"partialSuccess":{"rejectedSpans":0,"errorMessage":"SECRET-REMOTE-CONTENT"}}"#,
            "0 rejected spans",
        ),
        (r#"{"partialSuccess":{}}"#, ""),
        (r#"{"partialSuccess":{"rejectedSpans":"bad"}}"#, "acceptance is unknown"),
        ("not JSON", "acceptance is unknown"),
        (oversized.as_str(), "acceptance is unknown"),
        ("{}", ""),
    ] {
        let body = body.to_string();
        let (collector_port, seen) = listen(move |_| Resp::ok(body.clone())).await;
        let (model_port, _) = listen(|_| Resp::ok(answer("hi"))).await;
        let file = ported(
            "tests/trace/partial_success/partial_success.yml",
            &[("collector", collector_port), ("model", model_port)],
        );
        let result = a_graph(&[&file, "hi"], None).await;
        assert_eq!(result.code, 0, "{}", result.stderr);
        assert_eq!(seen.lock().unwrap().len(), 1, "never retry accepted data");
        assert!(!result.stderr.contains("SECRET-REMOTE-CONTENT"));
        if expected.is_empty() {
            assert!(result.stderr.is_empty(), "{}", result.stderr);
        } else {
            assert!(result.stderr.contains(expected), "{}", result.stderr);
            assert!(result.stderr.contains("not retried"));
        }
    }
}
