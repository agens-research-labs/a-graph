use crate::common::service::*;

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

/// A caller that does not record its trace (flag `00`): the run sends nothing to the collector, its log still carries
/// the trace of the caller, and the answer says `00`. A graph with no `trace` block answers `00` too.
#[tokio::test]
async fn trace_not_sent() {
    let folder = "target/test-service/trace_not_sent/default";
    let (log, untraced_log) = (format!("{folder}/run.log"), format!("{folder}/untraced.log"));
    let _ = std::fs::remove_file(&log);
    let _ = std::fs::remove_file(&untraced_log);
    let (model, _) = listen(|_| Resp::ok(answer("ok"))).await;
    let (collector, seen) = listen(|_| Resp::ok("{}")).await;
    let s = start(&config_path()).await;

    let yml = read("graph.yml", &[("model", model), ("collector", collector)]);
    let caller = format!("00-{TRACE}-00f067aa0ba902b7-00");
    let (status, traceparent, body) = s
        .post_traced("/run", &[&caller], json!({ "yml": yml, "message": "hi" }))
        .await;
    assert_eq!(status, 200, "{body}");
    assert!(seen.lock().unwrap().is_empty(), "nothing is sent to the collector");
    let start = log_lines(&log)
        .into_iter()
        .find(|l| l["code"] == "run.start")
        .expect("run.start logged");
    assert_eq!(start["trace_id"], TRACE, "the log carries the trace of the caller");
    let span = start["span_id"].as_str().unwrap();
    assert_eq!(traceparent.as_deref(), Some(format!("00-{TRACE}-{span}-00").as_str()));

    let yml = read("untraced.yml", &[("model", model)]);
    let caller = format!("00-{TRACE}-00f067aa0ba902b7-01");
    let (status, traceparent, body) = s
        .post_traced("/run", &[&caller], json!({ "yml": yml, "message": "hi" }))
        .await;
    assert_eq!(status, 200, "{body}");
    let start = log_lines(&untraced_log)
        .into_iter()
        .find(|l| l["code"] == "run.start")
        .expect("run.start logged");
    assert_eq!(start["trace_id"], TRACE, "with no `trace` block too");
    let span = start["span_id"].as_str().unwrap();
    assert_eq!(
        traceparent.as_deref(),
        Some(format!("00-{TRACE}-{span}-00").as_str()),
        "no `trace` block: the trace is not sent"
    );
}
