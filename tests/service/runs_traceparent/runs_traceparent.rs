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

/// The 202 of `POST /runs` gives the `traceparent` of the run before it runs: the span it names is the run that the
/// collector receives later, in the trace of the caller.
#[tokio::test]
async fn runs_traceparent() {
    let (model, _) = listen(|_| Resp::ok(answer("ok"))).await;
    let (collector, seen) = listen(|_| Resp::ok("{}")).await;
    let s = start(&config_path()).await;
    let yml = read("graph.yml", &[("model", model), ("collector", collector)]);
    let caller = format!("00-{TRACE}-00f067aa0ba902b7-01");
    let (status, traceparent, body) = s
        .post_traced("/runs", &[&caller], json!({ "yml": yml, "message": "hi" }))
        .await;
    assert_eq!(status, 202, "{body}");
    let traceparent = traceparent.expect("the 202 has a traceparent");
    let state = s.finished(body["run"].as_str().unwrap()).await;
    assert_eq!(state["state"], "done", "{state}");
    let spans = spans_of(&seen);
    let run = named(&spans, "run");
    assert_eq!(run["parentSpanId"], "00f067aa0ba902b7");
    assert_eq!(
        traceparent,
        format!("00-{TRACE}-{}-01", run["spanId"].as_str().unwrap())
    );
}
