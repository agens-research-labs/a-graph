use crate::common::service::*;
use crate::common::trace::{named, spans_of};

fn dir() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

fn read(name: &str, ports: &[(&str, u16)]) -> String {
    with_ports(&std::fs::read_to_string(dir().join(name)).unwrap(), ports)
}

fn config_path() -> String {
    dir().join("service.yml").to_string_lossy().to_string()
}

const TRACE: &str = "4bf92f3577b34da6a3ce929d0e0e4736";

/// A valid `traceparent` puts the run in the trace of the caller, under the span it names, and the answer, 200 or
/// 502, gives back the `traceparent` of the run: the same trace, the span of the run, the flag `01` (sent).
#[tokio::test]
async fn caller_trace_joined() {
    let (model, _) = listen(|r| {
        if r.body.contains("fail") {
            Resp::ok("not json")
        } else {
            Resp::ok(answer("ok"))
        }
    })
    .await;
    let (collector, seen) = listen(|_| Resp::ok("{}")).await;
    let s = start(&config_path()).await;
    let yml = read("graph.yml", &[("model", model), ("collector", collector)]);
    let caller = format!("00-{TRACE}-00f067aa0ba902b7-01");

    let (status, traceparent, body) = s
        .post_traced("/run", &[&caller], json!({ "yml": yml, "message": "hi" }))
        .await;
    assert_eq!(status, 200, "{body}");
    let spans = spans_of(&seen);
    let run = named(&spans, "run");
    assert_eq!(
        run["parentSpanId"], "00f067aa0ba902b7",
        "the run hangs from the span of the caller"
    );
    assert!(
        spans.iter().all(|s| s["traceId"] == TRACE),
        "every span is in the trace of the caller: {spans:?}"
    );
    assert_eq!(
        traceparent.as_deref(),
        Some(format!("00-{TRACE}-{}-01", run["spanId"].as_str().unwrap()).as_str())
    );

    // a run that fails is found the same way
    let before = spans.len();
    let (status, traceparent, body) = s
        .post_traced("/run", &[&caller], json!({ "yml": yml, "message": "fail" }))
        .await;
    assert_eq!(status, 502, "{body}");
    let spans = spans_of(&seen);
    let run = named(&spans[before..], "run");
    assert_eq!(run["traceId"], TRACE);
    assert_eq!(
        traceparent.as_deref(),
        Some(format!("00-{TRACE}-{}-01", run["spanId"].as_str().unwrap()).as_str())
    );
}
