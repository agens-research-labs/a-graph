use crate::common::service::*;

fn dir() -> &'static std::path::Path {
    std::path::Path::new(file!()).parent().unwrap()
}

#[allow(dead_code)]
fn read(name: &str) -> String {
    std::fs::read_to_string(dir().join(name)).unwrap()
}

fn config_path() -> String {
    dir().join("service.yml").to_string_lossy().to_string()
}

/// On `SIGTERM` a run that would last longer than the 30 seconds of the shutdown is stopped then, and still ends
/// as any run: its log gets the last event, with `service_stopped`. The service exits soon after, with 0.
#[cfg(unix)]
#[tokio::test]
async fn shutdown_stops() {
    let log = "target/test-service/shutdown_stops/default/run.log";
    let _ = std::fs::remove_file(log);
    let (_, seen) = serve_at_slowly(17411, std::time::Duration::from_secs(120), |_| Resp::ok(answer("too late"))).await;
    let mut s = start(&config_path(), 17412).await;
    let (status, body) = s.start_run(json!({ "yml": read("graph.yml"), "message": "how long?" })).await;
    assert_eq!(status, 202, "{body}");
    // the run is going on once the model is called: then the signal
    for _ in 0..100 {
        if !seen.lock().unwrap().is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert_eq!(seen.lock().unwrap().len(), 1, "the model was not called");
    let signalled = std::time::Instant::now();
    s.terminate();
    assert_eq!(s.exited(40).await, Some(0));
    assert!(signalled.elapsed() >= std::time::Duration::from_secs(30), "the run was stopped before the deadline: {:?}", signalled.elapsed());
    let lines = log_lines(log);
    let last = lines.last().unwrap_or_else(|| panic!("the log is empty"));
    assert_eq!(last["code"], "service_stopped", "{lines:?}");
}
