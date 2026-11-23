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

/// A `traceparent` that is not valid, two of them, or none: the run has a trace of its own, as before, and the answer
/// gives it back with the span of the run.
#[tokio::test]
async fn traceparent_ignored() {
    let (model, _) = listen(|_| Resp::ok(answer("ok"))).await;
    let (collector, seen) = listen(|_| Resp::ok("{}")).await;
    let s = start(&config_path()).await;
    let yml = read("graph.yml", &[("model", model), ("collector", collector)]);
    let valid = format!("00-{TRACE}-00f067aa0ba902b7-01");
    let upper = format!("00-{}-00f067aa0ba902b7-01", TRACE.to_uppercase());
    let cases: [Vec<&str>; 3] = [vec![&upper], vec![&valid, &valid], vec![]];
    for headers in &cases {
        let before = spans_of(&seen).len();
        let (status, traceparent, body) = s
            .post_traced("/run", headers, json!({ "yml": yml, "message": "hi" }))
            .await;
        assert_eq!(status, 200, "{body}");
        let spans = spans_of(&seen);
        let run = named(&spans[before..], "run");
        let trace = run["traceId"].as_str().unwrap();
        assert_ne!(trace, TRACE, "{headers:?}: the header is ignored");
        assert_eq!(trace.len(), 32);
        assert!(run.get("parentSpanId").is_none(), "{headers:?}: the run is the root");
        assert_eq!(
            traceparent.as_deref(),
            Some(format!("00-{trace}-{}-01", run["spanId"].as_str().unwrap()).as_str()),
            "{headers:?}"
        );
    }
}
